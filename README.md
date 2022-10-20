# Fluffl
[![Build status](https://github.com/K-C-DaCosta/fluffl/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/K-C-DaCosta/fluffl/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/fluffl)](https://crates.io/crates/fluffl)
[![Documentation](https://docs.rs/fluffl/badge.svg)](https://docs.rs/fluffl)


# what is fluffl?
fluffl is a media layer that provides an extremely simple, but cross-platform, interface between **desktop** and **wasm** targets.
Its built on top of the <a href="https://github.com/grovesNL/glow">glow</a> OpenGL bindings.

## why fluffl? 
If you need a *simple* layer/interface that provides audio,graphics, and maybe networking then this is the crate for you. 
Interface-wise its like SDL(you can use literally use* SDL if you select it) but it *doesn't* depend on the `wasm32-unknown-emscripten` target. The emscripten target is considered to be **deprecated** and is intended on being phased out last I checked. Instead, this crate uses the preferred `wasm32-unkown-unknown` target when building for the browser. 

## Supported Backends
- For the web it uses `WEBGL` and `WEBAUDIO`
- For desktop:
    - if `SDL2` is selected for windowing 
        - Audio options are:
            - `SDL2` but with AUDIO_SUBSYTEM enabled
    - if `GLUTIN` is slected for windowing
        - Audio options are:
            - `ALSA` - on linux 
            - `WASAPI` - on windows 

For desktop targets `GLUTIN` (for windowing) and native audio APIS **are chosen by default** since it doesn't require the program to link to `SDL2` dynamic libraries since `SDL2` may not be installed on a lot of machines we can avoid a link error on compile. `GLUTIN` also appears to use either use native libraries or directly interacts with operating-system specific windowing protocols (major protocols are X windowing protocol and Wayland on linux)  

## Using SDL2
If you *STILL* want to use SDL2 make sure its actually Installed

### Installing SDL2 On ubuntu
Just use apt to install:
```
sudo apt install libsdl2-dev
```

### Installing SDL2 On windows

Its slightly more complicated. IIRC, you have to either drag the sdl2.dll (you either download it off the offical website or compile it yourself) file to a special directory where the compiler sits or place it in the directory where the binary is. My directions are currently vague because my main OS is linux and I'd have to reconfig my KVM instance of windows to figure out exactly what to do again. Luckily you can just cross-compile. 

## Cross Compiling to windows on Linux (doesn't matter if you select SDL2 or not)

Using MinGW you can actually build for windows on linux.

On ubuntu first install mingw :
```
sudo apt install mingw-w64
```

Then use rustup to install the mingw toolchain
```
rustup target add x86_64-pc-windows-gnu
```
Finally to compile you program do:

```
cargo build --target=x86_64-pc-windows-gnu
```
The beauty of cross compilation is you can immediatly test the windows binary on your linux machine by running binary in `WINE` and it *just werks* (TM) . 

Wine chads... **I kneel**

## a simple example
Check the `examples` folder for runnable demos.
Wasm version of the examples are here: https://k-c-dacosta.github.io/wasm_bins/examples/audio_ex_1/

## A breakout clone (more complicated)
https://k-c-dacosta.github.io/wasm_bins/examples/brick_demo/

## How to run examples
- look in `./fluffl/examples` 
- pick a file you want to run (lets say we want to run brick_demo)
- In the terminal do:

```
cargo run --exmaple=brick_demo
``` 
## Update/Thoughts so far 
- I'm considering removing:
    - websocket module (tungstenite)
    - the Vorbis decoder (lewton) 
    - the mp3 decoder (puremp3)

This crate has over 20k lines. I have a hand-coded **GUI** ,**AudioMixer**, **linear-algebra** and **fixed-pont** libs built into the library that I'm considering splitting off into other crates.