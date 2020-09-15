# what is glue?
glue is a simple front end that provides an extremely simple, but cross-platform interface between desktop and wasm targets.

# why glue? 
I basically needed a generic way to write OpenGL for graphics, do audio processing and handle keyboard input etc for both WASM and Desktop targets, but there were no big crates that were as simple as SDL but buildable for both WASM32 and desktop. The rust-sdl2 project doesn't appear have good WASM support. Every SDL2 demo floating around on github I compile is broken in some way out of the box and also relies on the wasm32-unknown-emscripten target 
which appearently has larger binaries: https://kripken.github.io/blog/binaryen/2018/04/18/rust-emscripten.html.

# Status

## Update *[Mon sept 14 2020]*

Audio backends have been implemented for both desktop and web.
Examples now have instructions on how to run. 
The Desktop interface diverges slightly from the web interface. So work has to be done to make sure both implementations have consistent interfaces(which is the whole point of this project).

## Update *[???]*

Event handling, and very basic IO is implemented for both desktop and web targets, 
however, the audio backend for the web target is currently being worked on and is actually  close to being finished (the webaudio API sucks).  
The project, in its current state, can be used for any multimedia applications that dont need sound I guess, but other than that its still unusable.