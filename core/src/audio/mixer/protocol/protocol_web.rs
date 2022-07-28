use super::*;

pub use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    rc::Rc,
};

pub struct RequestQueue<T> {
    queue: Rc<RefCell<VecDeque<MixerRequest<T>>>>,
}

impl<T> Clone for RequestQueue<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
        }
    }
}

impl<T> RequestQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn try_lock<'a>(&'a self) -> Option<Ref<'a, VecDeque<MixerRequest<T>>>> {
        self.queue.try_borrow().ok()
    }

    pub fn try_lock_mut<'a>(&'a self) -> Option<RefMut<'a, VecDeque<MixerRequest<T>>>> {
        self.queue.try_borrow_mut().ok()
    }
}

#[derive(Clone)]
pub struct ResponseQueue {
    queue: Rc<RefCell<VecDeque<MixerResponse>>>,
}

impl ResponseQueue {
    pub fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }
    pub fn try_lock<'a>(&'a self) -> Option<Ref<'a, VecDeque<MixerResponse>>> {
        self.queue.try_borrow().ok()
    }

    pub fn try_lock_mut<'a>(&'a self) -> Option<RefMut<'a, VecDeque<MixerResponse>>> {
        self.queue.try_borrow_mut().ok()
    }
}
