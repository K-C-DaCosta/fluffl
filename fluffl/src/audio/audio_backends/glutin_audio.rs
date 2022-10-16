use super::{AudioDeviceCore, ConcreteSpecs};

cfg_if::cfg_if!{
    if #[cfg(target_os="linux")] {
        #[path ="./glutin_audio/glutin_alsa.rs"]
        mod glutin_audio_backend;
    }else if #[cfg(target_os="windows")] {
        #[path ="./glutin_audio/glutin_wasapi.rs"]
        mod glutin_audio_backend;
    }
}

pub use glutin_audio_backend::*; 