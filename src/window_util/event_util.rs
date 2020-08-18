pub mod constants; 

// #[cfg(not(feature="glutin"))]
// #[path="./event_util/sdl2_event.rs"]
// pub mod event;

// #[cfg(feature="glutin")]
// #[path="./event_util/glutin_event.rs"]
// pub mod event;  

use std::collections::VecDeque;
use constants::{EventKind};

/// A generic interface for events\
/// Unfortunately I needed a layer between SDL2 and other WASM+JS interfaces for the web build\
/// because the SDL2 crate pretty much only supports desktop platforms
pub struct GlueEvent{
    event_queue:VecDeque<EventKind>
}

impl GlueEvent{
    pub fn new()->Self{
        Self{
            event_queue:VecDeque::new(),
        }
    }
    pub fn push_event( &mut self, event:EventKind){
       self.event_queue.push_back(event); 
    }
    pub fn iter_mut(&mut self)->EventIter{
        EventIter{
            event:self,
        }
    }
}
pub struct EventIter<'a>{
    event:&'a mut GlueEvent,
}
impl <'a> Iterator for EventIter<'a>{
    type Item = EventKind;
    fn next(&mut self) -> Option<Self::Item> {
        self.event.event_queue.pop_front()
    }
}



