use super::*;

//the dektop implementation of sound is in the sdl2_audio module
#[cfg(not(all(target_family = "wasm", not(target_os = "wasi"))))]
#[path = "./protocol/protocol_desk.rs"]
pub mod platform_specifics;

//the wasm(javascript) implementation for sound playback is in the web_audio module
#[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
#[path = "./protocol/protocol_web.rs"]
pub mod platform_specifics;

pub use platform_specifics::*;

type Track = Box<dyn HasAudioStream>;

pub struct LocalRequestQueue {
    queue: VecDeque<MixerRequest>,
}
impl LocalRequestQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, req: MixerRequest) {
        self.queue.push_back(req);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

#[derive(Copy, Clone)]
pub enum OffsetKind {
    /// inserts track at t=begin+`offset`, where offset is in milliseconds
    Start { offset: u64 },
    /// inserts track at t=current+`offset`, where offset is in milliseconds
    Current { offset: i64 },
}
impl OffsetKind {
    /// ## Description
    /// Places the track at the current cursor position with no offset.\
    /// The track will begin playing almost immediatly after a request is made.
    pub fn current() -> Self {
        Self::default()
    }
}

impl Default for OffsetKind {
    fn default() -> Self {
        Self::Current { offset: 0 }
    }
}

pub enum MixerRequest {
    Seek(OffsetKind),
    /// the user invents a `TrackID` to associate with `Track`
    AddTrack(TrackID, OffsetKind, Track),
    RemoveTrack(TrackID),
    /// fetches the internal mixer time 
    FetchMixerTime,
    /// Send this to preform advanced mutations on a track
    /// or simply 
    MutateMixer(TrackID,fn(TrackID,&mut Mixer) -> MutatedResult<()>),
}

#[derive(Debug)]
pub enum AddTrackErr {
    TrackIdAlreadyExists(Track),
}

#[derive(Debug)]
pub enum RemoveTrackErr {
    TrackNotFound,
    TrackCurrentlyPlaying
}

#[derive(Debug)]
pub enum TrackMutatedErr {
    TrackNotFound,
}

#[derive(Debug)]
pub enum SeekErr {
    CursorOutOfBounds,
}

#[derive(Debug,Copy,Clone)]
pub enum MixerEventKind{
    /// the track has began playing for the first time
    TrackStarted(TrackID),
    /// the track has finished playing.
    TrackStopped(TrackID),
}

#[derive(Debug)]
pub enum MixerResponse {
    AddTrackStatus(TrackID, Result<(), AddTrackErr>),
    RemoveTrackStatus(TrackID, Result<Track, RemoveTrackErr>),
    /// the function was executed
    MixerMutatedStatus(TrackID,Result<(), TrackMutatedErr>),
    SeekStatus(Result<(), SeekErr>),
    /// ## Description
    /// The mixer will let the user know things when certain tracks start or are finised playing etc\ 
    MixerEvent(MixerEventKind),
    MixerTime(SampleTime),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TrackID {
    id: u64,
}
impl TrackID {
    pub fn from_u64(id: u64) -> Self {
        Self { id }
    }
    
    /// for when you dont care about mutating a specific track but still want
    /// to mutate the mixer   
    pub const fn null()->Self{
        Self{
            id:!0,
        }
    }
}

impl Default for TrackID{
    fn default() -> Self {
        Self::null()
    }
}
