use audio_ex1::*;

// Using fluffl on desktop will require to to setup an entry point in main.rs.
// Obviously, because you will want to target wasm, we need to do a little bit of conditional 
// compilation. Fluffl tries not to impose an async runtime on the user,in all examples we just use tokio. 
fn main() {
    //if desktop feature is selected compile code (lines 11-15)
    #[cfg(feature = "desktop")]
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _x = fluffl_main().await;
            println!("ERROR: {:?}", _x);
        });
    }
}
