pub mod constants; 
use std::collections::VecDeque;
pub use constants::*; 

/// A generic interface for events\
/// Unfortunately I needed a layer between SDL2 and other WASM+JS interfaces for the web build\
/// because the SDL2 crate pretty much only supports desktop platforms
pub struct FlufflEvent{
    event_queue:VecDeque<EventKind>
}

impl FlufflEvent{
    pub fn new()->Self{
        Self{
            event_queue:VecDeque::new(),
        }
    }

    pub fn push_event( &mut self, event:EventKind){
       self.event_queue.push_back(event); 
    }
    
    /// returns an iterator that iterates through event queue. 
    /// This iterator removes events as it walks through the queue. 
    pub fn flush_iter_mut(&mut self)->EventIter{
        EventIter{
            event:self,
        }
    }
}
pub struct EventIter<'a>{
    event:&'a mut FlufflEvent,
}
impl <'a> Iterator for EventIter<'a>{
    type Item = EventKind;
    fn next(&mut self) -> Option<Self::Item> {
        self.event.event_queue.pop_front()
    }
}



