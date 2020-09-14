use super::{GlueAudioDeviceCore, IntoWithArg};


use wasm_bindgen::prelude::*;
use wasm_bindgen::*;
use wasm_bindgen_futures::*; 
use web_sys::*;

use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

use crate::collections::linked_list::*;


static mut AUDIO_THREADS: Option<PackedLinkedList<js_sys::Function>> = None;

/// Inits a gobally managed pool of 'audio threads'(Theese threads are NOT executed concurrently).\
/// They are executed asynconously. These javascript functions are put on either the microtask or task queue of the hosts javascript engine\ 
/// When buffering or sound processing is needed the browser will allocated  allocated a time-slice for these functions to be executed asyncronously.
pub fn init_audio_threads(){
    unsafe{
        AUDIO_THREADS = Some(PackedLinkedList::new());
    }
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
    _marker: std::marker::PhantomData<(F, S)>,
}

impl<F, S> GlueAudioDeviceContext<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send,
    S: Send,
{
    pub fn new(core: GlueAudioDeviceCore<F, S>, audio_context: Arc<RefCell<GlueAudioContext>>) -> Self {
        let ctx = audio_context.clone();
        let temp_ctx = ctx.clone();
        let mut pcm_samples: Vec<u8> = Vec::new();
        let mut samples: Vec<f32> = Vec::new();
        let max_samples = 1024;
        let pump_list: Rc<RefCell<VecDeque<js_sys::Function>>> =
            Rc::new(RefCell::new(VecDeque::new()));

        let state = core.state.unwrap_or_else(||panic!("Error: Failed to create GlueAudioDevice!\n .with_state(..) not initalized!\n")) ;
        let state = Rc::new(RefCell::new(state));


        spawn_local( async { 

        });

        // if let Ok(data) = fetch {
        //     let mut play_time = ctx.borrow().ctx.current_time();

        //     let data = Rc::new(RefCell::new(data));

        //     let header = &data.borrow()[0..45];
        //     //I keep an explicit pointer that I update evertime I read the music data
        //     let mut byte_index = 0;

        //     let temp_data = data.clone();
        //     let process_raw_pcm = move || {
        //         samples.resize(max_samples, 0.0);
        //         pcm_samples.resize(max_samples * 2, 0);

        //         let mut raw_pcm = &temp_data.borrow()[byte_index..];
        //         // console_log!("play time = {}", play_time);

        //         while play_time - ctx.borrow().ctx.current_time() < 1. {
        //             let buffer = ctx
        //                 .borrow()
        //                 .ctx
        //                 .create_buffer(1, max_samples as u32, 22000.0f32)
        //                 .unwrap();
        //             let mut samples = buffer.get_channel_data(0).unwrap();

        //             raw_pcm.read(&mut pcm_samples).map(|bytes_read| {
        //                 let samples_read = bytes_read >> 1;
        //                 byte_index += bytes_read;
        //                 let pcm_array: &[TinyAudioSample] = unsafe {
        //                     slice::from_raw_parts(
        //                         pcm_samples.as_ptr() as *mut TinyAudioSample,
        //                         max_samples,
        //                     )
        //                 };
        //                 for i in 0..max_samples {
        //                     unsafe {
        //                         *samples.get_unchecked_mut(i) =
        //                             pcm_array.get_unchecked(i).channel[0] as f32 / 255.0;
        //                     }
        //                 }
        //             });

        //             buffer.copy_to_channel(&mut samples[..], 0).unwrap();

        //             let bsn = ctx.borrow().ctx.create_buffer_source().unwrap();
        //             bsn.set_buffer(Some(&buffer));

        //             //when this buffer finished playing, continue buffering
        //             let tpl = pump_list.clone();
        //             let continue_buffering = move || {
        //                 let cb = unsafe { CALLBACK.as_ref() };
        //                 let f = cb.unwrap();
        //                 let pump_list = tpl;
        //                 f.call0(&JsValue::null());
        //                 // because we know 'onended' had to have fired  we can remove the oldest buffer
        //                 // this buffer is likely to be garbage collected by the JS interpreter
        //                 pump_list.borrow_mut().pop_front();
        //                 // console_log!(
        //                 //     "callback triggered, pl size = {}",
        //                 //     pump_list.borrow().len()
        //                 // );
        //             };

        //             //wrap continue_buffering in boxed closure and convert to a Js Function
        //             let cb = Closure::once_into_js(continue_buffering)
        //                 .dyn_into::<js_sys::Function>()
        //                 .unwrap();

        //             pump_list.borrow_mut().push_back(cb);
        //             bsn.set_onended(pump_list.borrow().back());

        //             bsn.start_with_when(play_time).unwrap();

        //             play_time += max_samples as f64 / 22000.0;

        //             let node: AudioNode = bsn.dyn_into::<AudioNode>().unwrap();
        //             node.connect_with_audio_node(
        //                 &ctx.borrow().ctx.destination().dyn_into().unwrap(),
        //             );
        //             // console_log!("play time = {}", ctx.borrow().ctx.current_time() );
        //         }
        //     };

        //     let process_raw_pcm_closure =
        //         Closure::wrap(Box::new(process_raw_pcm) as Box<dyn FnMut()>)
        //             .into_js_value()
        //             .dyn_into::<js_sys::Function>()
        //             .unwrap();

        //     unsafe {
        //         //soud callback to global variable
        //         CALLBACK = Some(process_raw_pcm_closure);
        //         //execute sound thingy
        //         CALLBACK.as_ref().unwrap().call0(&JsValue::null());
        //     }
        // }
        panic!("not implemented");
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
    F: FnMut(&mut S, &mut [f32]) + Send + std::marker::Copy,
    S: Send,
{
    fn into_with(self, arg: Arc<RefCell<GlueAudioContext>>) -> GlueAudioDeviceContext<F, S> {
        GlueAudioDeviceContext::new(self, arg)
    }
}
