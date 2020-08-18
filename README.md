# what is g_lue?
g_lue is a simple front end that provides an extremely simple, but cross-platform interface  between desktop and wasm targets.

# why g_lue? 
I basically needed a generic way to write OpenGL for graphics, do audio processing and handle keyboard input etc for both WASM and Desktop targets. Basically I wanted SDL2 but builable for both WASM32 and desktop, however, the rust-sdl2 project doesn't have good WASM support. Every SDL2 demo floating around on github I compile is broken in some way out of the box. 

# Status
The project, in its current state, is unusable. Im  working on the WASM backend currently.  
