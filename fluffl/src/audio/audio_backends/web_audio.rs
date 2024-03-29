use super::{AudioDeviceCore, ConcreteSpecs};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// use wasm_bindgen_futures::*;
use web_sys::*;

use std::{cell::RefCell, rc::Rc};

use crate::{
    collections::{linked_list::*, Ptr},
    console::*,
    *,
};

struct ThreadState {
    play_time: f64,
    audio_thread: js_sys::Function,
}

// audio 'threads' are just JS functions that get put on
static mut AUDIO_THREADS: Option<LinkedList<Option<ThreadState>>> = None;

/// Inits a gobally managed pool of 'audio threads'(These threads are NOT executed in parallel).\
/// These javascript functions are put on either the microtask or task queue of the hosts javascript engine\
/// When buffering or sound processing is needed the browser will allocated  allocated a time-slice for these functions to be executed asyncronously.
pub fn init_audio_threads() {
    unsafe {
        AUDIO_THREADS = Some(LinkedList::new());
    }
}

fn get_audio_thread_list() -> &'static LinkedList<Option<ThreadState>> {
    unsafe { AUDIO_THREADS.as_ref().unwrap() }
}

fn get_audio_thread_list_mut() -> &'static mut LinkedList<Option<ThreadState>> {
    unsafe { AUDIO_THREADS.as_mut().unwrap() }
}

fn get_thread_state<'a>(thread_id: Ptr) -> Option<&'a ThreadState> {
    get_audio_thread_list()[thread_id]
        .get_data()
        .unwrap()
        .as_ref()
}

fn get_thread_state_mut<'a>(thread_id: Ptr) -> Option<&'a mut ThreadState> {
    get_audio_thread_list_mut()[thread_id]
        .get_data_mut()
        .unwrap()
        .as_mut()
}

fn can_buffer(thread_id: Ptr, buffer_time: f64, audio_context: FlufflAudioContext) -> bool {
    let difference =
        get_thread_state(thread_id).unwrap().play_time - audio_context.ctx.current_time();
    difference < buffer_time
}

#[derive(Clone)]
pub struct FlufflAudioContext {
    pub ctx: AudioContext,
}

impl FlufflAudioContext {
    pub fn new() -> Self {
        let ctx = AudioContext::new().unwrap();
        Self { ctx }
    }
}

impl Default for FlufflAudioContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FlufflAudioContext {
    fn drop(&mut self) {
        let _ = self.ctx.close();
    }
}

pub struct FlufflAudioDeviceContext<F, S> {
    glue_callback: F,
    state: Rc<RefCell<S>>,
    thread_id: Ptr,
    audio_context: FlufflAudioContext,
}

impl<F, S> Clone for FlufflAudioDeviceContext<F, S>
where
    F: Copy,
{
    fn clone(&self) -> Self {
        Self {
            glue_callback: self.glue_callback,
            state: self.state.clone(),
            thread_id: self.thread_id,
            audio_context: self.audio_context.clone(),
        }
    }
}

pub struct AudioBufferPool {
    buffer_pool: Vec<AudioBuffer>,
    free_buffers: Vec<u64>,
}
impl AudioBufferPool {
    fn new(
        audio_context: &AudioContext,
        channels: u32,
        buffer_size: u32,
        sample_rate: f32,
    ) -> Self {
        Self {
            buffer_pool: (0..32)
                .map(|_| {
                    audio_context
                        .create_buffer(channels, buffer_size, sample_rate)
                        .unwrap()
                })
                .collect(),
            free_buffers: (0..32).collect(),
        }
    }

    fn get_unused_audio_buffer(
        &mut self,
        audio_context: &AudioContext,
        channels: u32,
        buffer_size: u32,
        sample_rate: f32,
    ) -> u64 {
        if self.free_buffers.is_empty() {
            let new_buffer = audio_context
                .create_buffer(channels, buffer_size, sample_rate)
                .unwrap();
            self.buffer_pool.push(new_buffer);
            (self.buffer_pool.len() - 1) as u64
        } else {
            self.free_buffers.pop().unwrap()
        }
    }

    fn free_audio_buffer(&mut self, buffer_index: u64) {
        self.free_buffers.push(buffer_index);
    }
}

impl std::ops::Index<u64> for AudioBufferPool {
    type Output = AudioBuffer;
    fn index(&self, index: u64) -> &Self::Output {
        &self.buffer_pool[index as usize]
    }
}

impl<F, S> FlufflAudioDeviceContext<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + Copy + 'static,
    S: 'static,
{
    pub fn new(mut core: AudioDeviceCore<F, S>, audio_context: FlufflAudioContext) -> Self {
        // the time (in seconds) to buffer ahead to avoid choppy playback
        const BUFFER_TIME: f64 = 0.3;

        let state = core.state.take().unwrap_or_else(|| {
            panic!("Error: Failed to create FlufflAudioDevice!\n .with_state(..) not initalized!\n")
        });

        let mut glue_callback = core.callback();
        let state = Rc::new(RefCell::new(state));

        let ConcreteSpecs {
            sample_rate,
            channels,
            buffer_size,
        } = core.desired_specs.make_concrete();

        let audio_context_dup = audio_context.clone();

        let get_state_ptr = |state: Rc<RefCell<S>>| (&mut *state.borrow_mut()) as *mut S;
        let state_ptr = get_state_ptr(state.clone());

        //this buffer contains INTERLEAVED pcm in 32-bit IEEE-754 floating point precision
        //however the webaudio api demanands each channel be submitted seperately,so this routine will have
        //to manually split the PCM codes after glue_callback(...) is called.
        let mut sample_callback_buffer = Vec::new();

        //this is the buffer that I will be using to represent non-interleaved samples per chennel
        let mut sample_buffer_for_channel = Vec::new();

        //make sure that allocation happens up front
        sample_buffer_for_channel.resize(buffer_size, 0f32);

        //allocate uninitalized audio 'thread' to front of linked list
        get_audio_thread_list_mut().push_front(None);

        // get pointer to the front
        let thread_id = get_audio_thread_list().get_front();

        // allocate a bunch of empty buffers upfront before playing any music
        // buffer pool should let us recycle buffers to some degree
        let buffer_pool: Rc<RefCell<AudioBufferPool>> =
            Rc::new(RefCell::new(AudioBufferPool::new(
                &audio_context.ctx,
                channels as u32,
                buffer_size as u32,
                sample_rate as f32,
            )));

        let process_raw_pcm = move || {
            //buffer for BUFFER_TIME into the future
            while can_buffer(thread_id, BUFFER_TIME, audio_context.clone()) {
                // // console_log!("blocks counted = {}\n", audio_blocks_pushed);
                // let buffer_count = buffer_pool.borrow().buffer_pool.len();
                // console_log!("pool size ={}", buffer_count);

                let web_audio_buffer_ptr = buffer_pool.borrow_mut().get_unused_audio_buffer(
                    &audio_context.ctx,
                    channels as u32,
                    buffer_size as u32,
                    sample_rate as f32,
                );

                //clear buffers before calling the callback
                sample_callback_buffer.resize(buffer_size * channels, 0f32);

                //call the callback provided by the user
                let state_ref = unsafe { &mut *state_ptr };
                glue_callback(state_ref, &mut sample_callback_buffer[..]);

                //de-interleave samples into a buffer with samples associated with just a single channel
                for channel_index in 0..channels {
                    //clear the buffer holding PCM for a specific channel
                    sample_buffer_for_channel.clear();

                    //collect samples for channel 'channel_index'
                    for k in 0..sample_callback_buffer.len() / channels {
                        let sample_index = k * channels + channel_index;
                        sample_buffer_for_channel.push(sample_callback_buffer[sample_index]);
                    }

                    //copy the callback buffer to the web_audio_buffer
                    buffer_pool.borrow()[web_audio_buffer_ptr]
                        .copy_to_channel(&sample_buffer_for_channel[..], channel_index as i32)
                        .unwrap();
                }

                let web_audio_buffer_source_node =
                    audio_context.ctx.create_buffer_source().unwrap();

                {
                    let web_audio_buffer = &buffer_pool.borrow()[web_audio_buffer_ptr];
                    web_audio_buffer_source_node.set_buffer(Some(web_audio_buffer));
                }

                // This function is fired whenever 'onended' event is fired
                // continue_buffering() literally just resumes the 'thread'
                let buffer_pool_ptr = buffer_pool.clone();
                let continue_buffering = move || {
                    buffer_pool_ptr
                        .borrow_mut()
                        .free_audio_buffer(web_audio_buffer_ptr);

                    if let Some(ThreadState { audio_thread, .. }) = get_audio_thread_list()
                        [thread_id]
                        .get_data()
                        .and_then(|data| data.as_ref())
                    {
                        let _ = audio_thread.call0(&JsValue::null());
                    }
                };

                //wrap continue_buffering in boxed closure and convert to a Js Function
                let cb = Closure::once_into_js(continue_buffering)
                    .dyn_into::<js_sys::Function>()
                    .unwrap();

                // when this buffer finished playing resume buffering
                // web_audio_buffer_source_node.set_onended(pump_list.borrow().back());
                web_audio_buffer_source_node.set_onended(Some(&cb));

                // prepare to connect audio node to destination (MUST be done before playing sound)
                // by casting down to &AudioNode
                let node: &AudioNode = web_audio_buffer_source_node.dyn_ref::<AudioNode>().unwrap();

                // Connecting source to destination happens here. I don't bother
                // checking the connection results. don't care about it right now,
                // this could go wrong in the future.
                let _connect_result = node
                    .connect_with_audio_node(&audio_context.ctx.destination().dyn_into().unwrap());

                // play the buffer at time t=play_time
                web_audio_buffer_source_node
                    .start_with_when(get_thread_state(thread_id).unwrap().play_time)
                    .unwrap();

                //because each chunk of samples takes some time,DT,to play  I have to
                //increment play_time by that amount. DT in this case is equal to : buffer_size/sample_rate
                if let Some(thread_state) = get_thread_state_mut(thread_id) {
                    thread_state.play_time += buffer_size as f64 / sample_rate as f64;
                }
            }
        };

        // The audio 'thread' get initalized here
        let process_raw_pcm_closure = Closure::wrap(Box::new(process_raw_pcm) as Box<dyn FnMut()>)
            .into_js_value()
            .dyn_into::<js_sys::Function>()
            .unwrap();

        //initalize 'audio thread'
        if let Some(data) = get_audio_thread_list_mut()[thread_id].get_data_mut() {
            *data = Some(ThreadState {
                play_time: 0.0,
                audio_thread: process_raw_pcm_closure,
            });
        }

        Self {
            state,
            glue_callback,
            thread_id,
            audio_context: audio_context_dup,
        }
    }

    pub fn modify_state<CBF>(&self, mut cb: CBF)
    where
        CBF: FnMut(Option<&mut S>) -> Option<()>,
    {
        if let Ok(mut state_ptr) = self.state.try_borrow_mut() {
            let state_ref = &mut *state_ptr;
            let _ = cb(Some(state_ref));
        }
    }

    pub fn resume(&self) {
        let thread_id = self.thread_id;
        //begin 'audio thread'
        if let Some(data) = get_audio_thread_list_mut()[thread_id].get_data_mut() {
            data.as_mut().map(
                |ThreadState {
                     play_time,
                     audio_thread,
                 }| {
                    // really need to make sure that play_time is updated before playing
                    *play_time = self.audio_context.ctx.current_time();
                    // this line,once executed, actually starts the thread
                    audio_thread.call0(&JsValue::null())
                },
            );
        }
    }

    pub fn pause(&self) {
        unimplemented!("pause not implmented, not really a high priority as this functionality can be implemented somewhere else");
    }
}

impl<F, S> Drop for FlufflAudioDeviceContext<F, S> {
    fn drop(&mut self) {
        let thread_id = self.thread_id;

        //release audio 'thread' from list Option<js_function> gets dropped
        let result: Option<Option<ThreadState>> = get_audio_thread_list_mut().remove(thread_id);
        if let Some(Some(func)) = result {
            console_log!("drop func\n");
            std::mem::drop(func);
        }

        // // forceably free the audio state (necessary because Closure::into_js_value() leaks )
        // // and WILL cause a dangling pointer
        // unsafe {
        //     mem::ManuallyDrop::drop(&mut *self.state.borrow_mut());
        // }

        let state_count = Rc::strong_count(&self.state);
        console_log!(
            "CONTEXT[ id={} ] IS ABOUT TO BE DROPPED, state_count = {}\n",
            thread_id,
            state_count
        );

        //if this context gets dropped then the state of type 'S' should also get dropped freeing
        //up some ram
    }
}
