// use web_sys::{Performance, Window};

pub struct Instant {
    t0: f64,
}
impl Instant {
    pub fn now() -> Self {
        Self {
            t0: web_sys::window()
                .expect("failed to fetch window object")
                .performance()
                .expect("performance unavailable")
                .now(),
        }
    }
    pub fn elapsed(&self) -> Duration {
        Duration { t0: self.t0 }
    }
}

pub struct Duration {
    t0: f64,
}

impl Duration {
    pub fn as_millis(&self) -> u128 {
        let t1 = web_sys::window()
            .expect("failed to fetch window object")
            .performance()
            .expect("performance unavailable")
            .now();
        (t1 - self.t0) as u128
    }
}
