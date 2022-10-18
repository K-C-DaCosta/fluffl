use super::*;
use crate::{console::*, *};
use std::sync::{Arc, Mutex};
use std::{
    thread,
    time::{Duration, Instant},
};

use windows::{
    core::*,
    Win32::{
        Devices::FunctionDiscovery::*,
        Media::{
            Audio::{eRender, IMMDeviceEnumerator, MMDeviceEnumerator, WAVEFORMATEX, *},
            KernelStreaming::*,
            Multimedia::*,
        },
        System::Com::*,
        UI::Shell::PropertiesSystem::PROPERTYKEY,
    },
};

#[derive(Clone, Default)]
pub struct FlufflAudioContext {
    /*
    This struct is not really needed for WASAPI EITHER
    but for SDL2 and WEBAUDIO a struct like this
    is needed
    */
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum DeviceState {
    Playing,
    Paused,
}

pub struct FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send + 'static,
    State: Send + 'static,
{
    fluffl_audio_device: Arc<Mutex<FlufflAudioDevice<Callback, State>>>,
    state: Arc<Mutex<DeviceState>>,
}

impl<Callback, State> Clone for FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    fn clone(&self) -> Self {
        Self {
            fluffl_audio_device: self.fluffl_audio_device.clone(),
            state: self.state.clone(),
        }
    }
}

impl<Callback, State> FlufflAudioDeviceContext<Callback, State>
where
    Callback: FnMut(&mut State, &mut [f32]) + Copy + Send,
    State: Send,
{
    /// creates a platform-agnostic FlufflAudioDevice
    pub fn new(core: AudioDeviceCore<Callback, State>, _actx: FlufflAudioContext) -> Self {
        let audio_device = Arc::new(Mutex::new(FlufflAudioDevice { core }));
        Self {
            fluffl_audio_device: audio_device,
            state: Arc::new(Mutex::new(DeviceState::Paused)),
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
        println!("resume called");
        let requested_specs = self
            .fluffl_audio_device
            .lock()
            .unwrap()
            .core
            .desired_specs
            .make_concrete();

        //spawn dedacated thread for funneling state PCM into WASAPI
        let ctx = self.clone();
        thread::spawn(move || unsafe {
            if let Err(e) = Self::wasapi_resume_thread(requested_specs, ctx) {
                eprintln!("err ={}", e);
            }
        });
    }

    /// pauses the device
    pub fn pause(&self) {
        // this simply just signals the thread to stop playing and clean up after itself
        *self.state.lock().unwrap() = DeviceState::Paused;
    }

    unsafe fn wasapi_resume_thread(mut requested_specs: ConcreteSpecs, ctx: Self) -> Result<()> {

        if let Ok(mut state) =ctx.state.lock(){
            if let DeviceState::Paused = *state{
                *state = DeviceState::Playing;
            }else{
                println!("already playing");
                return Ok(());
            }
        }

        CoInitializeEx(None, COINIT_MULTITHREADED).expect("co initalize failed");

        let clsid_enumerator = &MMDeviceEnumerator as *const GUID;
        let enumerator = CoCreateInstance::<_, IMMDeviceEnumerator>(
            clsid_enumerator,
            InParam::null(),
            CLSCTX_ALL,
        )
        .expect("co create failed");
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let audio_client = device.Activate::<IAudioClient>(CLSCTX_ALL, None)?;

        let client_requesting_mono = requested_specs.channels == 1;
        if client_requesting_mono {
            //tell wasapi to make it stereo
            requested_specs.channels = 2;
        }

        let requested_device_format =
            request_device_format(&audio_client, requested_specs).expect("request failed");

        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            0,
            //allocate 30ms worth of data to buffer ahead by
            (100_000_000 * 3) / 100,
            0,
            (&requested_device_format) as *const _ as *const _,
            None,
        )?;

        let render_client: IAudioRenderClient = audio_client.GetService()?;

        match client_requesting_mono {
            true => Self::read_mono_from_state_and_mix_to_stereo(
                ctx,
                &audio_client,
                &render_client,
                requested_specs,
            ),
            false => Self::read_requested_format_from_state_no_mixing(
                ctx,
                &audio_client,
                &render_client,
                requested_specs,
            ),
        }
    }



    /// WASAPI works differently from ALSA in that If you request mono,
    /// WASAPI wont automatically mix that mono signal to stereo, instead,
    /// it pumps the audio to only ONE speaker (not what I want at all).
    /// My current solution is to mix the mono signal to stereo myself. I suspect that
    /// WASAPI may be able to preform the mixing itself but I'm currently unaware of this functionality.
    /// I'll keep investigating the microsoft docs, but for now this works.
    unsafe fn read_mono_from_state_and_mix_to_stereo(
        ctx: Self,
        audio_client: &IAudioClient,
        render_client: &IAudioRenderClient,
        requested_specs: ConcreteSpecs,
    ) -> Result<()> {
        /*
            
        */
        let buffer_frame_count = audio_client.GetBufferSize()?;
        audio_client.Start()?;
        let mut can_sleep = None;
        let frequency = requested_specs.sample_rate as u64;
        let mut mono_buffer = vec![0f32; requested_specs.buffer_size];

        loop {
            if let Some(num_frames_written) = can_sleep.take() {
                let duration_buffered_in_nanos = (num_frames_written * 1_000_000_000) / frequency;
                std::thread::sleep(Duration::from_nanos(duration_buffered_in_nanos / 4));
            }
            let num_frames_padding = audio_client.GetCurrentPadding()?;
            let num_frames_available = buffer_frame_count - num_frames_padding;
            if num_frames_available < requested_specs.buffer_size as u32 {
                // the internal buffer is full so  just busy wait
                // for more frames to become available so i don't
                // overwrite what's already there
                can_sleep = Some((buffer_frame_count - num_frames_available) as u64);
                continue;
            }
            if let Ok(mut device) = ctx.fluffl_audio_device.try_lock() {
                let mut callback = device.callback();
                let state = device.state().expect("state not initalized");

                // ask the render_client to expose a pointer to the buffer of PCM data
                let pcm_buffer = render_client.GetBuffer(requested_specs.buffer_size as u32)?;

                // cast the pcm_buffer to a float slice
                let pcm_buffer_slice = std::slice::from_raw_parts_mut(
                    pcm_buffer as *mut f32,
                    requested_specs.buffer_size * requested_specs.channels as usize,
                );

                // let the callback write data to the WASAPI buffer
                let _ = callback(state, &mut mono_buffer);

                // mix mono to stereo
                pcm_buffer_slice
                    .chunks_mut(2)
                    .zip(mono_buffer.iter())
                    .flat_map(|(pout, &pin)| pout.iter_mut().map(move |val| (val, pin)))
                    .for_each(|(pout, pin)| {
                        *pout = pin;
                    });

                // let WASAPI know im finished writing to the buffer, so it can enqueue what I wrote
                render_client.ReleaseBuffer(requested_specs.buffer_size as u32, 0)?;
            }
        }
        Ok(())
    }

    unsafe fn read_requested_format_from_state_no_mixing(
        ctx: Self,
        audio_client: &IAudioClient,
        render_client: &IAudioRenderClient,
        requested_specs: ConcreteSpecs,
    ) -> Result<()> {
        let buffer_frame_count = audio_client.GetBufferSize()?;
        audio_client.Start()?;
        let mut can_sleep = None;
        let frequency = requested_specs.sample_rate as u64;
        loop {
            if let Some(num_frames_written) = can_sleep.take() {
                let duration_buffered_in_nanos = (num_frames_written * 1_000_000_000) / frequency;
                std::thread::sleep(Duration::from_nanos(duration_buffered_in_nanos / 4));
            }
            let num_frames_padding = audio_client.GetCurrentPadding()?;
            let num_frames_available = buffer_frame_count - num_frames_padding;
            if num_frames_available < requested_specs.buffer_size as u32 {
                // the internal buffer is full so  just busy wait
                // for more frames to become available so i don't
                // overwrite what's already there
                can_sleep = Some((buffer_frame_count - num_frames_available) as u64);
                continue;
            }
            Self::pump_data(ctx.clone(), &render_client, requested_specs)?;
        }
        Ok(())
    }

    unsafe fn pump_data(
        ctx: Self,
        render_client: &IAudioRenderClient,
        requested_specs: ConcreteSpecs,
    ) -> Result<()> {
        if let Ok(mut device) = ctx.fluffl_audio_device.try_lock() {
            let mut callback = device.callback();
            let state = device.state().expect("state not initalized");

            // ask the render_client to expose a pointer to the buffer of PCM data
            let pcm_buffer = render_client.GetBuffer(requested_specs.buffer_size as u32)?;

            // cast the pcm_buffer to a float slice
            let pcm_buffer_slice = std::slice::from_raw_parts_mut(
                pcm_buffer as *mut f32,
                requested_specs.buffer_size * requested_specs.channels as usize,
            );

            // let the callback write data to the WASAPI buffer
            let _ = callback(state, pcm_buffer_slice);

            // let WASAPI know im finished writing to the buffer, so it can enqueue what I wrote
            render_client.ReleaseBuffer(requested_specs.buffer_size as u32, 0)?;
        }
        Ok(())
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

pub unsafe fn request_device_format(
    audio_client: &IAudioClient,
    requested_specs: ConcreteSpecs,
) -> Result<WAVEFORMATEXTENSIBLE> {
    /*
        According to MS docs:
        "The method(GetMixFormat) always uses a WAVEFORMATEXTENSIBLE structure, instead of a stand-alone WAVEFORMATEX structure, to specify the format."
        Source:https://learn.microsoft.com/en-us/windows/win32/coreaudio/device-formats
    */
    // let format_buffer = audio_client.GetMixFormat()? as *mut WAVEFORMATEXTENSIBLE;

    // let device_format_header = &mut *format_buffer;

    // print_format_header("default", device_format_header);

    let mut requested_device_format = requested_specs.to_wasapi();

    print_format_header("requested", &requested_device_format);

    let mut closest_match = std::ptr::null_mut::<WAVEFORMATEX>();
    let result = audio_client.IsFormatSupported(
        AUDCLNT_SHAREMODE_SHARED,
        ((&requested_device_format) as *const WAVEFORMATEXTENSIBLE) as *const _,
        Some((&mut closest_match) as *mut _),
    );

    if result.is_ok() {
        println!("WASAPI: requested format is supported ✅");
        // *device_format_header = requested_device_format;
    } else {
        println!("WASAPI: format is not supported ❌");
        println!("WASAPI: checking for closest device...");
        if closest_match.is_null() {
            eprintln!("WASAPI: no closest match found 🤦🤦 can't play audio");
            return Err(Error::from_win32());
        }
        println!("WASAPI: Requested format not found BUT the next best thing is:");
        print_format_header(
            "closest",
            &mut **(closest_match as *mut *mut WAVEFORMATEXTENSIBLE),
        );
    }

    Ok(requested_device_format)
}

pub unsafe fn print_format_header(dev_name: &str, hdr: &WAVEFORMATEXTENSIBLE) {
    println!("== START {dev_name} device format ==");
    println!("nSamplesPerSec = {}", { hdr.Format.nSamplesPerSec });
    println!("nChannels = {}", { hdr.Format.nChannels });
    println!("nBlockAlign = {}", { hdr.Format.nBlockAlign });
    println!("nBitsPerSample = {}", { hdr.Format.wBitsPerSample });
    println!("nAverageBytesPerSec = {}", { hdr.Format.nAvgBytesPerSec });
    println!("cbSize ={}", { hdr.Format.cbSize });
    println!("Samples.wValidBitsPerSample = {}", {
        hdr.Samples.wValidBitsPerSample
    });
    println!("Samples.wSamplesPerBlock = {}", {
        hdr.Samples.wSamplesPerBlock
    });
    println!("Samples.wReserved = {}", { hdr.Samples.wReserved });
    println!("dwChannelMask = {}", { hdr.dwChannelMask });
    println!("SubFormat = {}", to_readable_subformat(hdr.SubFormat));
    println!("== END {dev_name} device format ==");
}

pub fn to_readable_subformat(subformat: GUID) -> &'static str {
    match subformat {
        KSDATAFORMAT_SUBTYPE_PCM => "integer PCM ",
        KSDATAFORMAT_SUBTYPE_IEEE_FLOAT => "IEEE-754 PCM",
        _ => "unknown/unsupported subformat",
    }
}

impl ConcreteSpecs {
    /// convert to a WASAPI pod that describes essentially the same thing
    #[allow(non_snake_case)]
    pub fn to_wasapi(self) -> WAVEFORMATEXTENSIBLE {
        let requested_specs = self;
        let wBitsPerSample = 32;
        let nBlockAlign = (requested_specs.channels as u16 * wBitsPerSample) / 8;
        WAVEFORMATEXTENSIBLE {
            Format: WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_EXTENSIBLE as u16,
                nChannels: requested_specs.channels as u16,
                nSamplesPerSec: requested_specs.sample_rate,
                nBlockAlign,
                wBitsPerSample,
                nAvgBytesPerSec: requested_specs.sample_rate * nBlockAlign as u32,
                cbSize: 22,
            },
            Samples: WAVEFORMATEXTENSIBLE_0 {
                wValidBitsPerSample: 32,
            },
            dwChannelMask: (0..requested_specs.channels)
                .map(|speaker_bit| 1 << speaker_bit)
                .sum(),
            SubFormat: KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
        }
    }
}
