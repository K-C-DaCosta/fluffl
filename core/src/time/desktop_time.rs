//literally just a wrapper for std time on desktop
use std::time;


pub struct Instant{
    inst:time::Instant,
}

impl Instant{
    pub fn now()->Self{
        Self{
            inst:time::Instant::now(),
        }
    }
    pub fn elapsed(&self) -> Duration{
        Duration { dur: self.inst.elapsed() }
    }

}

pub struct Duration{
    dur:time::Duration,
}
impl Duration{
    pub fn as_millis(&self)-> u128{
        self.dur.as_millis()
    }
}