# Status
## Update *[Wed nov 4 2020]* 
- Added websocket client functionality 
- Working on canvas resizing issue on web side of things 
Other than that i just need to work on documentation and examples
Crates.io upload will be soon  

## Update *[Mon sept 14 2020]*

Audio backends have been implemented for both desktop and web.
Examples now have instructions on how to run. 
The Desktop interface diverges slightly from the web interface. So work has to be done to make sure both implementations have consistent interfaces(which is the whole point of this project).

## Update *[???]*

Event handling, and very basic IO is implemented for both desktop and web targets, 
however, the audio backend for the web target is currently being worked on and is actually  close to being finished (the webaudio API sucks).  
The project, in its current state, can be used for any multimedia applications that dont need sound I guess, but other than that its still unusable.