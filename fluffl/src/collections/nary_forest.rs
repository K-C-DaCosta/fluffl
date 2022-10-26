use super::Ptr;

use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

#[derive(Serialize, Deserialize)]
pub struct NaryNode<T> {
    pub parent: Ptr,
    pub data: Option<T>,
    pub children: Vec<Ptr>,
}

impl<T> NaryNode<T> {
    pub fn new() -> NaryNode<T> {
        NaryNode {
            parent: Ptr::null(),
            data: None,
            children: Vec::new(),
        }
    }
    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }
}

impl<T> Default for NaryNode<T> {
    fn default() -> Self {
        Self::new()
    }
}

///Custom N-ary Tree implemented with vector-backed memory
#[derive(Serialize, Deserialize)]
pub struct NaryForest<T> {
    pub root_list: Vec<Ptr>,
    pub pool: Ptr,
    pub memory: Vec<NaryNode<T>>,
}

impl<T> NaryForest<T>
where
    Self: Index<Ptr, Output = NaryNode<T>> + IndexMut<Ptr>,
{
    pub fn new() -> NaryForest<T> {
        NaryForest {
            root_list: Vec::new(),
            pool: Ptr::null(),
            memory: Vec::new(),
        }
    }
    pub fn allocate(&mut self, val: T) -> Ptr {
        if self.pool == Ptr::null() {
            self.memory.push(NaryNode::new().with_data(val));
            Ptr::from(self.memory.len() - 1)
        } else {
            let pool_node = self.pool;
            self.pool = self[pool_node].children[0];
            self[pool_node].children.clear();
            pool_node
        }
    }

    #[allow(dead_code)]
    pub fn free(&mut self, node: Ptr) {
        if node == Ptr::null() {
            return;
        }
        if self.pool != Ptr::null() {
            let old_pool = self.pool;
            self[node].children.clear();
            self[node].children.push(old_pool);
        }
        self.pool = node;
    }

    #[allow(dead_code)]
    pub fn allocate_node(&mut self, node: NaryNode<T>) -> Ptr {
        if self.pool == Ptr::null() {
            self.memory.push(node);
            Ptr::from(self.memory.len() - 1)
        } else {
            let pool_node = self.pool;
            self.pool = self[pool_node].children[0];
            self[pool_node].children.clear();
            pool_node
        }
    }
    pub fn add_child(&mut self, parent: Ptr, child: Ptr) {
        self[parent].children.push(child);
        self[child].parent = parent;
    }
}

impl<T> Default for NaryForest<T>
where
    Self: Index<Ptr, Output = NaryNode<T>> + IndexMut<Ptr>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Ptr> for NaryForest<T> {
    type Output = NaryNode<T>;

    fn index(&self, ptr: Ptr) -> &Self::Output {
        self.memory.get(ptr.as_usize()).unwrap()
    }
}

impl<T> IndexMut<Ptr> for NaryForest<T> {
    fn index_mut(&mut self, ptr: Ptr) -> &mut Self::Output {
        self.memory.get_mut(ptr.as_usize()).unwrap()
    }
}
