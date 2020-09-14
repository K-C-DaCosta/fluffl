use super::{GlueAudioDeviceCore, IntoWithArg};

use wasm_bindgen::prelude::*;
use wasm_bindgen::*;
use wasm_bindgen_futures::*;
use web_sys::*;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

use crate::collections::linked_list::*;

static mut AUDIO_THREADS: Option<LinkedList<Option<js_sys::Function>>> = None;

/// Inits a gobally managed pool of 'audio threads'(Theese threads are NOT executed concurrently).\
/// They are executed asynconously. These javascript functions are put on either the microtask or task queue of the hosts javascript engine\
/// When buffering or sound processing is needed the browser will allocated  allocated a time-slice for these functions to be executed asyncronously.
pub fn init_audio_threads() {
    unsafe {
        AUDIO_THREADS = Some(LinkedList::new());
    }
}

fn get_thread_list() -> &'static LinkedList<Option<js_sys::Function>> {
    unsafe { AUDIO_THREADS.as_ref().unwrap() }
}

fn get_thread_list_mut() -> &'static mut LinkedList<Option<js_sys::Function>> {
    unsafe { AUDIO_THREADS.as_mut().unwrap() }
}

pub struct GlueAudioContext {
    pub ctx: AudioContext,
}

impl GlueAudioContext {
    pub fn new() -> Self {
        let ctx = AudioContext::new().unwrap();
        Self { ctx }
    }
}

impl Drop for GlueAudioContext {
    fn drop(&mut self) {
        let _ = self.ctx.close();
    }
}

pub struct GlueAudioDeviceContext<F, S> {
    state: Rc<RefCell<S>>,
    _marker: std::marker::PhantomData<(F, S)>,
}

impl<F, S> GlueAudioDeviceContext<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send + 'static,
    S: Send + 'static,
{
    pub fn new(
        mut core: GlueAudioDeviceCore<F, S>,
        audio_context: Arc<RefCell<GlueAudioContext>>,
    ) -> Self {
        const BUFFER_TIME: f64 = 1.0;

        let pump_list: Rc<RefCell<VecDeque<js_sys::Function>>> =
            Rc::new(RefCell::new(VecDeque::new()));

        let state = core.state.take().unwrap_or_else(|| {
            panic!("Error: Failed to create GlueAudioDevice!\n .with_state(..) not initalized!\n")
        });

        let mut glue_callback = core.callback();
        let state = Rc::new(RefCell::new(state));
        let context_state = state.clone();
        let (sample_rate, channels, buffer_size) = core.desired_specs.get_specs();
        let mut play_time = audio_context.borrow().ctx.current_time();

        //this buffer contains INTERLEAVED pcm in 32-bit IEEE-754 floating point precision
        //however the webaudio api demanands each channel be submitted seperately,so this routine will have
        //to manually split the PCM codes after glue_callback(...) is called.
        let mut sample_callback_buffer = Vec::new();
        let mut sample_buffer_for_channel = Vec::new();

        //make sure that allocation happens up front
        sample_buffer_for_channel.resize(buffer_size, 0f32);

        //allocate uninitalized 'thread'
        get_thread_list_mut().push_front(None);
        let thread_id = get_thread_list().get_front();

        let process_raw_pcm = move || {
            //buffer for one second into the future
            while play_time - audio_context.borrow().ctx.current_time() < BUFFER_TIME {
                let web_audio_buffer = audio_context
                    .borrow()
                    .ctx
                    .create_buffer(1, buffer_size as u32, sample_rate as f32)
                    .unwrap();

                let mut web_audio_samples = web_audio_buffer.get_channel_data(0).unwrap();

                //clear buffers before calling the callback
                sample_callback_buffer.resize(buffer_size * channels, 0f32);

                //call the callback provided by the user
                glue_callback(&mut *state.borrow_mut(), &mut sample_callback_buffer[..]);

                //'demux' interleaved samples into a buffer with samples associated with just a single channel
                for channel in 0..channels {
                    for &sample in sample_callback_buffer.iter().step_by(channel) {
                        sample_buffer_for_channel.push(sample);
                    }
                    //copy the callback buffer to the web_audio_buffer
                    web_audio_buffer
                        .copy_to_channel(&mut sample_callback_buffer[..], channel as i32)
                        .unwrap();
                    //clear the buffer holding PCM for a specific channel
                    sample_buffer_for_channel.clear();
                }

                let web_audio_buffer_source_node =
                    audio_context.borrow().ctx.create_buffer_source().unwrap();

                web_audio_buffer_source_node.set_buffer(Some(&web_audio_buffer));

                // //when this buffer finished playing, continue buffering
                let tpl = pump_list.clone();
                let continue_buffering = move || {
                    let f:&js_sys::Function = get_thread_list()[thread_id].get_data().as_ref().unwrap();
                    let pump_list = tpl;
                    f.call0(&JsValue::null());
                    // because we know 'onended' had to have fired  we can remove the oldest buffer
                    // this buffer is likely to be garbage collected by the JS interpreter
                    pump_list.borrow_mut().pop_front();
                    // console_log!(
                    //     "callback triggered, pl size = {}",
                    //     pump_list.borrow().len()
                    // );
                };

                // //wrap continue_buffering in boxed closure and convert to a Js Function
                let cb = Closure::once_into_js(continue_buffering)
                    .dyn_into::<js_sys::Function>()
                    .unwrap();

                pump_list.borrow_mut().push_back(cb);
                web_audio_buffer_source_node.set_onended(pump_list.borrow().back());
                web_audio_buffer_source_node.start_with_when(play_time).unwrap();

                play_time += buffer_size as f64 / sample_rate as f64;

                let node: AudioNode = web_audio_buffer_source_node.dyn_into::<AudioNode>().unwrap();
                node.connect_with_audio_node(&audio_context.borrow().ctx.destination().dyn_into().unwrap());
                // console_log!("play time = {}", ctx.borrow().ctx.current_time() );
            }
        };
        // The audio 'thread' get initalized here

        let process_raw_pcm_closure = Closure::wrap(Box::new(process_raw_pcm) as Box<dyn FnMut()>)
            .into_js_value()
            .dyn_into::<js_sys::Function>()
            .unwrap();

        //initalize 'audio thread'
        *get_thread_list_mut()[thread_id].get_data_mut() = Some(process_raw_pcm_closure);

        //begin 'audio thread'
        get_thread_list_mut()[thread_id]
            .get_data_mut()
            .as_ref()
            .unwrap()
            .call0(&JsValue::null());

        Self {
            state: context_state,
            _marker: std::marker::PhantomData::default(),
        }
    }

    pub fn modify_state<CBF>(&self, mut cb: CBF)
    where
        CBF: FnMut(Option<&mut S>),
    {
        panic!("not implemented");
    }

    pub fn resume(&self) {
        panic!("not implemented");
    }

    pub fn pause(&self) {
        panic!("not implemented");
    }
}

pub struct GlueAudioDevice<F, S> {
    core: GlueAudioDeviceCore<F, S>,
}

impl<F, S> GlueAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    pub fn callback(&self) -> F {
        panic!("not implemented");
    }

    pub fn state(&mut self) -> Option<&mut S> {
        panic!("not implemented");
    }
}

impl<F, S> IntoWithArg<GlueAudioDeviceContext<F, S>, Arc<RefCell<GlueAudioContext>>>
    for GlueAudioDeviceCore<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + Send + std::marker::Copy + 'static,
    S: Send + 'static,
{
    fn into_with(self, arg: Arc<RefCell<GlueAudioContext>>) -> GlueAudioDeviceContext<F, S> {
        GlueAudioDeviceContext::new(self, arg)
    }
}
