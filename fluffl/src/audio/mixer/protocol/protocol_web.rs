use super::*;

pub use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    rc::Rc,
};

pub struct RequestQueuePtr {
    queue: Rc<RefCell<VecDeque<MixerRequest>>>,
}

impl Clone for RequestQueuePtr {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

impl RequestQueuePtr {
    pub fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn submit_requests(&self, requests: &mut LocalRequestQueue) {
        let queue_that_only_user_has_access_to = requests;
        if let Ok(mut queue_that_mixer_has_access_to) = self.queue.try_borrow_mut() {
            while let Some(req) = queue_that_only_user_has_access_to.queue.pop_front() {
                queue_that_mixer_has_access_to.push_back(req)
            }
        }
    }

    pub fn lock(&self) -> Option<RefMut<'_, VecDeque<MixerRequest>>> {
        self.queue.try_borrow_mut().ok()
    }
}
impl Default for RequestQueuePtr {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ResponseQueuePtr {
    queue: Rc<RefCell<VecDeque<MixerResponse>>>,
}

impl ResponseQueuePtr {
    pub fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn lock(&self) -> Option<RefMut<'_, VecDeque<MixerResponse>>> {
        self.queue.try_borrow_mut().ok()
    }

    pub fn recieve_responses(&self) -> impl Iterator<Item = MixerResponse> + '_ {
        self.queue
            .try_borrow_mut()
            .ok()
            .map(|guard| DequeueAndRemove::new(Some(guard)))
            .unwrap_or_else(|| DequeueAndRemove::new(None))
    }
}

impl Default for ResponseQueuePtr {
    fn default() -> Self {
        Self::new()
    }
}
pub struct DequeueAndRemove<'a, T> {
    queue: Option<RefMut<'a, VecDeque<T>>>,
}
impl<'a, T> DequeueAndRemove<'a, T> {
    pub fn new(queue: Option<RefMut<'a, VecDeque<T>>>) -> Self {
        Self { queue }
    }
}
impl<'a, T> Iterator for DequeueAndRemove<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.as_mut().and_then(|queue| queue.pop_front())
    }
}
