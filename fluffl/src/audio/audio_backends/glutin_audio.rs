use super::AudioDeviceCore;
use std::sync::{Arc, Mutex};

use std::{ffi::CString, io::Write, thread};

use alsa::{
    pcm::{Access, Format, HwParams, State, PCM},
    Output, ValueOr,
};

#[derive(Clone, Default)]
pub struct FlufflAudioContext {
    /*
    not really needed for ALSA 
    but for SDL2 and WEBAUDIO a struct like this 
    is needed
    */
}

pub struct FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send + 'static,
{
    fluffl_audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
}

impl<Callback, State> Clone for FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    fn clone(&self) -> Self {
        Self {
            fluffl_audio_device: self.fluffl_audio_device.clone(),
        }
    }
}

impl<Callback, State> FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    /// creates a platform-agnostic FlufflAudioDevice
    pub fn new(
        core: AudioDeviceCore<Callback, State>,
        actx: FlufflAudioContext,
    ) -> FlufflAudioDeviceContext<Callback, State> {
        
        unimplemented!("An FlufflAudioDeviceContext is currently in development");


        let audio_device = Arc::new(Mutex::new(FlufflAudioDevice { core }));

        let callback = FlufflCallback {
            audio_device: audio_device.clone(),
        };

        //select the default audio device
        let pcm = alsa::pcm::PCM::new("default", alsa::Direction::Playback, true)
            .expect("alsa: default device failed");

        //configure device with core's specifications
        let specs = audio_device.lock().unwrap().core.desired_specs;


        let hwp = alsa::pcm::HwParams::any(&pcm).expect("hw params failed");
        hwp.set_channels(1).expect("set_channels(..) failed");
        hwp.set_rate(44100, ValueOr::Nearest)
            .expect("set_format(..) failed");
        hwp.set_format(Format::float())
            .expect("set_format(..) failed");
        hwp.set_access(Access::RWInterleaved)
            .expect("set_access(..) failed");
        pcm.hw_params(&hwp).unwrap();

        let hwp = pcm.hw_params_current().unwrap();
        let swp = pcm.sw_params_current().unwrap();
        swp.set_start_threshold(hwp.get_buffer_size().unwrap())
            .unwrap();
        pcm.sw_params(&swp).unwrap();

        println!(
            "PCM status: {:?}, {:?}",
            pcm.state(),
            pcm.hw_params_current().unwrap()
        );
        let mut outp = Output::buffer_open().unwrap();
        pcm.dump(&mut outp).unwrap();
        println!("== PCM dump ==\n{}", outp);

        Self {
            fluffl_audio_device: audio_device,
        }
    }

    /// ## Description
    /// Allows the user to modify state through a callback
    /// ### Comments
    /// If I can't easily return the value to code higher up in the stack,
    /// the next best thing is pass a callback to the value  
    pub fn modify_state<ModifyCallback>(&self, mut cb: ModifyCallback)
    where
        ModifyCallback: FnMut(Option<&mut State>) -> Option<()>,
    {
        let mut lck = self.fluffl_audio_device.lock().unwrap();
        let device = &mut *lck;
        let s = device.core.state.as_mut();
        let _ = cb(s);
    }
    /// resumes the device
    pub fn resume(&self) {
        unimplemented!("resume(..)")
    }

    /// pauses the device
    pub fn pause(&self) {
        unimplemented!("pause(..) unimplemented ")
    }
}

pub struct FlufflAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send + 'static,
    S: Send,
{
    core: AudioDeviceCore<F, S>,
}

impl<F, S> FlufflAudioDevice<F, S>
where
    F: FnMut(&mut S, &mut [f32]) + std::marker::Copy + Send + 'static,
    S: Send + 'static,
{
    pub fn callback(&self) -> F {
        self.core.callback()
    }

    pub fn state(&mut self) -> Option<&mut S> {
        self.core.state.as_mut()
    }
}

pub struct FlufflCallback<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send,
{
    audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
}
