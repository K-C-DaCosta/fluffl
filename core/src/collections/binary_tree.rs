#![allow(dead_code)]

use std::ops::{Index, IndexMut};
use super::Ptr; 


#[derive(Copy, Clone)]
pub struct BinNode<T> {
    pub data: Option<T>,
    pub parent: Ptr,
    pub children: [Ptr; 2],
}
impl<T> BinNode<T> {
    pub fn new() -> Self {
        Self {
            data: None,
            parent: Ptr::null(),
            children: [Ptr::null(); 2],
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children[0] == Ptr::null() && self.children[1] == Ptr::null()
    }

    pub fn from_data(data: T) -> Self {
        Self {
            data: Some(data),
            parent: Ptr::null(),
            children: [Ptr::null(); 2],
        }
    }

    pub fn with_parent(mut self, parent: Ptr) -> Self {
        self.parent = parent;
        self
    }
}

pub struct BinaryTree<T> {
    root: Ptr,
    node: Vec<BinNode<T>>,
    pool: Ptr,
}

impl<T> BinaryTree<T> {

    pub fn new() -> Self {
        Self {
            root: Ptr::null(),
            node: Vec::new(),
            pool: Ptr::null(),
        }
    }
    pub fn nodes(&self)->&[BinNode<T>]{
        self.node.as_slice()
    }

    pub fn root(&self) -> Ptr {
        self.root
    }
    pub fn set_root(&mut self, root: Ptr) {
        self.root = root;
    }

    pub fn allocate(&mut self, parent: Ptr, item: T) -> Ptr {
        if self.pool == Ptr::null() {
            self.node.push(BinNode::from_data(item).with_parent(parent));
            Ptr::from(self.node.len() - 1)
        } else {
            let old_pool = self.pool;
            let new_node = self[old_pool].children[0];

            //initalize node
            self[old_pool].children = [Ptr::null(); 2];
            self[old_pool].parent = parent;
            self[old_pool].data = Some(item);

            //update pool pointer
            self.pool = new_node;

            old_pool
        }
    }

    pub fn free(&mut self, ptr: Ptr) -> Option<T> {
        let data = self[ptr].data.take();
        self[ptr].parent = Ptr::null();
        self[ptr].children = [self.pool, Ptr::null()];
        self.pool = ptr;
        data
    }
}
impl<T> Index<Ptr> for BinaryTree<T> {
    type Output = BinNode<T>;
    fn index(&self, index: Ptr) -> &Self::Output {
        &self.node[index.idx as usize]
    }
}
impl<T> IndexMut<Ptr> for BinaryTree<T> {
    fn index_mut(&mut self, index: Ptr) -> &mut Self::Output {
        &mut self.node[index.idx as usize]
    }
}
