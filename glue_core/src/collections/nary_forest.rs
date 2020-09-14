static NULL: u32 = !0;
type Ptr = u32;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct NaryNode<T> {
    pub parent: Ptr,
    pub data: Option<T>,
    pub children: Vec<Ptr>,
}

impl<T> NaryNode<T> {
    pub fn new() -> NaryNode<T> {
        NaryNode {
            parent: NULL,
            data: None,
            children: Vec::new(),
        }
    }
    pub fn with_data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
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
    Self: Index<u32, Output = NaryNode<T>> + IndexMut<u32>,
{
    pub fn new() -> NaryForest<T> {
        NaryForest {
            root_list: Vec::new(),
            pool: NULL,
            memory: Vec::new(),
        }
    }
    pub fn allocate(&mut self, val: T) -> Ptr {
        if self.pool == NULL {
            self.memory.push(NaryNode::new().with_data(val));
            (self.memory.len() - 1) as u32
        } else {
            let pool_node = self.pool;
            self.pool = self[pool_node].children[0];
            self[pool_node].children.clear();
            pool_node
        }
    }

    pub fn free(&mut self, node: Ptr) {
        if node == NULL {
            return;
        }
        if self.pool != NULL {
            let old_pool = self.pool;
            self[node].children.clear();
            self[node].children.push(old_pool);
        }
        self.pool = node;
    }

    pub fn allocate_node(&mut self, node: NaryNode<T>) -> Ptr {
        if self.pool == NULL {
            self.memory.push(node);
            (self.memory.len() - 1) as u32
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

impl<T> Index<u32> for NaryForest<T> {
    type Output = NaryNode<T>;

    fn index(&self, ptr: u32) -> &Self::Output {
        self.memory.get(ptr as usize).unwrap()
    }
}

impl<T> IndexMut<u32> for NaryForest<T> {
    fn index_mut(&mut self, ptr: u32) -> &mut Self::Output {
        self.memory.get_mut(ptr as usize).unwrap()
    }
}
