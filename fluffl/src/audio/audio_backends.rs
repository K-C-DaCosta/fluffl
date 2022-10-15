use super::*;

cfg_if::cfg_if! {

    if #[cfg(all(not(all(target_family = "wasm", not(target_os = "wasi"))), feature="sdl2"  ))]  {
        /// This just forwards to the sdl audio backend
        #[path ="./audio_backends/sdl2_audio.rs"]
        mod audio_backend;
    }else if #[cfg(all(not(all(target_family = "wasm", not(target_os = "wasi"))), feature="glutin"  ))]{
        /// i call the module "glutin_audio" however, glutin actually has no audio backend like SDL does.
        /// "glutin" is to stay consistent with the feature name
        #[path ="./audio_backends/glutin_audio.rs"]
        mod audio_backend;
    }else if #[cfg(all(target_family = "wasm", not(target_os = "wasi")))] {
        //web implementation selected here
        #[path = "./audio_backends/web_audio.rs"]
        mod audio_backend;
    }
}

pub use audio_backend::*;
