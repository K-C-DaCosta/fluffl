# what is fluffl?
fluffl is a media layer that  provides an extremely simple, but cross-platform, interface between **desktop** and **wasm** targets.
Its built on top of the <a href="https://github.com/grovesNL/glow">glow</a> interface, so it has good opengl support and should have
decent support for low-level audio programming, on both desktop and web.

## why fluffl? 
If you need a *simple* layer that provides audio,graphics, and maybe networking then this is the crate for you. 
Think of it like SDL but more primitive. 

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
