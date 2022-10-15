use super::*;

// cfg_if::cfg_if! {
//     if #[cfg(feature="be_sdl2")]  {
//         #[path ="./desktop_window/sdl2_window.rs"]
//         mod sdl2_window;
//         pub use sdl2_window::*;
//     }else if #[cfg(feature="be_glutin")] {
//         #[path ="./desktop_window/glutin_window.rs"]
//         mod glutin_window;
//         pub use glutin_window::*;
//     }
// }

cfg_if::cfg_if! {
    if #[cfg(all(not(all(target_family = "wasm", not(target_os = "wasi"))), feature="sdl2"  ))]  {
        /// This just forwards to the sdl audio backend
        #[path ="./window_backends/sdl2_window.rs"]
        mod window_backend;
    }else if #[cfg(all(not(all(target_family = "wasm", not(target_os = "wasi"))), feature="glutin"  ))]{
        /// i call the module "glutin_audio" however, glutin actually has no audio backend like SDL does.
        /// "glutin" is to stay consistent with the feature name
        #[path ="./window_backends/glutin_window.rs"]
        mod window_backend;
    }else if #[cfg(all(target_family = "wasm", not(target_os = "wasi")))] {
        //web implementation selected here
        #[path = "./window_backends/web_window.rs"]
        mod window_backend;
    }
}

pub use window_backend::*;
