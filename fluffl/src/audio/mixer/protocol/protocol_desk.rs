use super::*;

pub use std::{
    borrow::BorrowMut,
    collections::VecDeque,
    iter,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};


pub struct RequestQueuePtr {
    queue_ptr: Arc<Mutex<VecDeque<MixerRequest>>>,
}

impl RequestQueuePtr {
    pub fn new() -> Self {
        Self {
            queue_ptr: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn submit_requests(&self, requests: &mut LocalRequestQueue) {
        let queue_that_only_user_has_access_to = requests;
        if let Some(mut queue_that_mixer_has_access_to) = self.queue_ptr.try_lock().ok() {
            while let Some(req) = queue_that_only_user_has_access_to.queue.pop_front() {
                queue_that_mixer_has_access_to.borrow_mut().push_back(req)
            }
        }
    }

    pub fn lock<'a>(&'a self)->Option<MutexGuard<'a,VecDeque<MixerRequest>>>{
        self.queue_ptr.try_lock().ok()
    }
}

impl Clone for RequestQueuePtr {
    fn clone(&self) -> Self {
        Self {
            queue_ptr: self.queue_ptr.clone(),
        }
    }
}

pub struct ResponseQueuePtr {
    queue_ptr: Arc<Mutex<VecDeque<MixerResponse>>>,
}

impl ResponseQueuePtr {
    pub fn new() -> Self {
        Self {
            queue_ptr: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub fn recieve_responses(&self) -> impl Iterator<Item = MixerResponse> + '_ {
        self.queue_ptr
            .try_lock()
            .ok()
            .map(|guard| DequeueAndRemove::new(Some(guard)))
            .unwrap_or(DequeueAndRemove::new(None))
    }
    pub fn lock<'a>(&'a self)->Option<MutexGuard<'a,VecDeque<MixerResponse>>>{
        self.queue_ptr.try_lock().ok()
    }
}
impl Clone for ResponseQueuePtr {
    fn clone(&self) -> Self {
        Self {
            queue_ptr: self.queue_ptr.clone(),
        }
    }
}

pub struct DequeueAndRemove<'a, T> {
    queue: Option<MutexGuard<'a, VecDeque<T>>>,
}
impl<'a, T> DequeueAndRemove<'a, T> {
    pub fn new(queue: Option<MutexGuard<'a, VecDeque<T>>>) -> Self {
        Self { queue }
    }
}
impl<'a, T> Iterator for DequeueAndRemove<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue
            .as_mut()
            .and_then(|queue| queue.borrow_mut().pop_front())
    }
}
