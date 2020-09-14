# what is g_lue?
g_lue is a simple front end that provides an extremely simple, but cross-platform interface  between desktop and wasm targets.

# why g_lue? 
I basically needed a generic way to write OpenGL for graphics, do audio processing and handle keyboard input etc for both WASM and Desktop targets. Basically I wanted SDL2 but buildable for both WASM32 and desktop, however, the rust-sdl2 project doesn't have good WASM support. Every SDL2 demo floating around on github I compile is broken in some way out of the box. 

# Status
Event handling, and very basic IO is implemented for both desktop and web targets, 
however, the audio backend for the web target is currently being worked on and is actually  close to being finished (the webaudio API sucks).  
The project, in its current state, can be used for any multimedia applications that dont need sound I guess, but other than that its still unusable. 

