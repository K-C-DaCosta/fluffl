use audio_ex1::*;
#[allow(unused_imports)]
use fluffl::prelude::*;

pub fn main() {
    // This is the wasm entry point
    #[cfg(feature = "web")]{
        //this is optional, but gives you better error in the browser
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        //this actually gets the ball rolling
        spawn_local(async move {
            let _ = fluffl_main().await;
        });
    }
    
    // This is the desktop entry point
    // Fluffl tries not to impose an async runtime on the user,in all examples we just use tokio.
    #[cfg(feature = "desktop")]
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = fluffl_main().await;
        });
    }
}
