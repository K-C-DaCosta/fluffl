use std::ops::*;





/// The underlying memory is an array of pointers. Indirection performance penalties will apply\
/// 'T' can be pretty much anything.
#[allow(dead_code)]
pub type LinkedList<T> = DoublyLinkedList<OptionNode<T>>;

/// The underlying memory is a packed array of structs.\
/// This version of linked list is much more compact in memory and therefore has  better locality of reference.
/// This should only work if T is Copy
#[allow(dead_code)]
pub type PackedLinkedList<T> = DoublyLinkedList<Node<T>>;

/// Linked list operations I consider 'core' 
pub trait LLNodeCoreOps {
    fn get_children(&self) -> &[u32; 2];
    fn get_children_mut(&mut self) -> &mut [u32; 2];
    fn nullify(&mut self) {
        self.get_children_mut().iter_mut().for_each(|e| *e = !0);
    }
}
/// This is functionality every node should have
pub trait LLNodeOps<T>: Default {
    fn width_data(self, raw_data: T) -> Self;
    fn get_data(&self)->Option<&T>;
    fn get_data_mut(&mut self)->Option<&mut T>;  
}
/// Defines some higher order operations for a linked list 
pub trait LLOps<NodeType, DataType>
where
    NodeType: LLNodeOps<DataType> + LLNodeCoreOps,
{
    /// In this implementation 'memory' is just a vector.  This module implements what is referred to as a\
    /// 'vector-backed' linked list.
    fn get_memory(&mut self) -> &Vec<NodeType>;
    /// Returns a pointer to the pool
    fn get_pool(&self) -> u32;
    /// Returns a pointer to the rear dll 
    fn get_rear(&self) -> u32;
    /// Returns a pointer to the from of the dll
    fn get_front(&self) -> u32;
    /// returns the length of the dll 
    fn len(&self) -> u32;

    /// returns a mutable pointer to memory \
    /// Even though this is considered safe in rust, I would prefren manual manipulation \
    /// to be done by code in this module \
    unsafe fn get_memory_mut(&mut self) -> &mut Vec<NodeType>;

    /// returns a mutable refrence to the pool pointer for external manipulation
    unsafe fn get_pool_mut(&mut self) -> &mut u32;
    
    /// inserts a node to the left or right of location `cur_node` in "memmory" \
    /// `dir` =  0  when inserting to the left of cur_node \
    /// `dir` =  1  when inserting to the right of cur_node 
    fn insert(&mut self, cur_node: u32, dir: usize, data: DataType);

    /// removes a node at location `cur_node` in "memmory"
    fn remove(&mut self, cur_node: u32) -> Option<DataType>;
    
    /// allocates a new node
    fn allocate(&mut self, data: DataType) -> u32;

    fn push_front(&mut self, data: DataType) {
        self.insert(self.get_front(), 0, data);
    }

    fn pop_front(&mut self) -> Option<DataType> {
        self.remove(self.get_front())
    }

    fn push_rear(&mut self, data: DataType) {
        self.insert(self.get_rear(), 1, data);
    }

    fn pop_rear(&mut self) -> Option<DataType> {
        self.remove(self.get_rear())
    }
    
    /// free node at location `node`
    fn free(&mut self, node: u32) {
        if self.get_pool() == !0 {
            unsafe {
                *self.get_pool_mut() = node;
                self.get_memory_mut()[node as usize].nullify();
            }
        } else {
            unsafe {
                self.get_memory_mut()[node as usize].nullify();
                self.get_memory_mut()[node as usize].get_children_mut()[0] = self.get_pool();
                *self.get_pool_mut() = node;
            }
        }
    }
}

pub struct OptionNode<T> {
    data: Option<T>,
    children: [u32; 2],
}

impl<T> LLNodeCoreOps for OptionNode<T> {
    fn get_children(&self) -> &[u32; 2] {
        &self.children
    }
    fn get_children_mut(&mut self) -> &mut [u32; 2] {
        &mut self.children
    }
}

impl<T> LLNodeOps<T> for OptionNode<T> {
    fn width_data(self, raw_data: T) -> Self {
        Self {
            data: Some(raw_data),
            children: self.children,
        }
    }
    fn get_data(&self) ->Option<&T> {
        self.data.as_ref()
    }
    fn get_data_mut(&mut self) ->Option<&mut T> {
        self.data.as_mut()
    }
}

impl<T> Default for OptionNode<T> {
    fn default() -> Self {
        Self {
            data: None,
            children: [0; 2],
        }
    }
}

pub struct Node<T> {
    data: T,
    children: [u32; 2],
}

impl<T> LLNodeCoreOps for Node<T>
where
    T: Copy + Default,
{
    fn get_children(&self) -> &[u32; 2] {
        &self.children
    }
    fn get_children_mut(&mut self) -> &mut [u32; 2] {
        &mut self.children
    }
}

impl<T> LLNodeOps<T> for Node<T>
where
    T: Copy + Default,
{
    fn width_data(self, raw_data: T) -> Self {
        Self {
            data: raw_data,
            children: self.children,
        }
    }
    fn get_data(&self) ->Option<&T> {
        Some(&self.data)
    }
    fn get_data_mut(&mut self) ->Option<&mut T> {
        Some(&mut self.data)
    }
}

impl<T> Default for Node<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            data: T::default(),
            children: [0; 2],
        }
    }
}

pub struct DoublyLinkedList<NodeType> {
    memory: Vec<NodeType>,
    pub front: u32,
    pub rear: u32,
    pub pool: u32,
    pub len: u32,
}

pub struct DLLNodeIterator<LinkedList> {
    dll: LinkedList,
    node: u32,
    len: u32,
}

impl<'a, NodeType> Iterator for DLLNodeIterator<&'a DoublyLinkedList<NodeType>>
where
    NodeType: LLNodeCoreOps,
{
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let old_node = self.node;
            self.node = self.dll[old_node].get_children()[1];
            self.len -= 1;
            Some(self.node)
        } else {
            None
        }
    }
}
#[allow(dead_code)]
impl<NodeType> DoublyLinkedList<NodeType> {
    pub fn new() -> Self {
        Self {
            memory: Vec::new(),
            front: !0,
            rear: !0,
            pool: !0,
            len: 0,
        }
    }
}
#[allow(dead_code)]
impl<NodeType> DoublyLinkedList<NodeType>
where
    NodeType: LLNodeCoreOps,
{
    pub fn node_index_iter(&self) -> impl Iterator<Item = u32> + '_ {
        let node = self.front;
        let len = self.len;
        DLLNodeIterator {
            dll: self,
            node,
            len,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &NodeType> {
        self.node_index_iter().map(move |index| &self[index])
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NodeType> {
        let mem_ptr = self.memory.as_mut_ptr();
        self.node_index_iter()
            .map(move |index| unsafe { &mut *mem_ptr.offset(index as isize) })
    }
}

impl<T> LLOps<OptionNode<T>, T> for DoublyLinkedList<OptionNode<T>> {
    fn get_memory(&mut self) -> &Vec<OptionNode<T>> {
        &self.memory
    }

    fn get_pool(&self) -> u32 {
        self.pool
    }

    unsafe fn get_memory_mut(&mut self) -> &mut Vec<OptionNode<T>> {
        &mut self.memory
    }

    unsafe fn get_pool_mut(&mut self) -> &mut u32 {
        &mut self.pool
    }

    fn get_rear(&self) -> u32 {
        self.rear
    }
    fn get_front(&self) -> u32 {
        self.front
    }
    fn len(&self) -> u32 {
        self.len
    }

    fn insert(&mut self, cur_node: u32, dir: usize, data: T) {
        if self.len == 0 {
            let new_node = self.allocate(data);
            self[new_node].children[0] = new_node;
            self[new_node].children[1] = new_node;
            self.front = new_node;
            self.rear = new_node;
        } else {
            let new_node = self.allocate(data);
            let adj_node = self[cur_node].children[dir];

            self[cur_node].children[dir] = new_node;
            self[new_node].children[1 - dir] = cur_node;

            self[adj_node].children[1 - dir] = new_node;
            self[new_node].children[dir] = adj_node;

            if cur_node == self.front && dir == 0 {
                self.front = new_node;
            }

            if cur_node == self.rear && dir == 1 {
                self.rear = new_node;
            }
        }
        self.len += 1;
    }

    fn remove(&mut self, cur_node: u32) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let ln = self[cur_node].children[0];
            let rn = self[cur_node].children[1];
            self[ln].children[1] = rn;
            self[rn].children[0] = ln;
            let item = self[cur_node].data.take();
            self.free(cur_node);

            if cur_node == self.front {
                self.front = rn;
            }

            if cur_node == self.rear {
                self.rear = ln;
            }

            item
        }
    }

    fn allocate(&mut self, data: T) -> u32 {
        if self.pool == !0 {
            self.memory.push(OptionNode::default().width_data(data));
            self.memory.len() as u32 - 1
        } else {
            let old_pool = self.pool;
            let new_pool = self[old_pool].children[0];
            self[old_pool].data = Some(data);
            self[old_pool].nullify();
            self.pool = new_pool;
            old_pool
        }
    }
}

impl<T> LLOps<Node<T>, T> for DoublyLinkedList<Node<T>>
where
    T: Default + Copy,
{
    fn get_memory(&mut self) -> &Vec<Node<T>> {
        &self.memory
    }
    fn get_pool(&self) -> u32 {
        self.pool
    }
    fn get_rear(&self) -> u32 {
        self.rear
    }
    fn get_front(&self) -> u32 {
        self.front
    }
    fn len(&self) -> u32 {
        self.len
    }

    unsafe fn get_memory_mut(&mut self) -> &mut Vec<Node<T>> {
        &mut self.memory
    }

    unsafe fn get_pool_mut(&mut self) -> &mut u32 {
        &mut self.pool
    }

    fn insert(&mut self, cur_node: u32, dir: usize, data: T) {
        if self.len == 0 {
            let new_node = self.allocate(data);
            self[new_node].children[0] = new_node;
            self[new_node].children[1] = new_node;
            self.front = new_node;
            self.rear = new_node;
        } else {
            let new_node = self.allocate(data);
            let adj_node = self[cur_node].children[dir];

            self[cur_node].children[dir] = new_node;
            self[new_node].children[1 - dir] = cur_node;

            self[adj_node].children[1 - dir] = new_node;
            self[new_node].children[dir] = adj_node;

            if cur_node == self.front && dir == 0 {
                self.front = new_node;
            }

            if cur_node == self.rear && dir == 1 {
                self.rear = new_node;
            }
        }
        self.len += 1;
    }

    fn remove(&mut self, cur_node: u32) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let ln = self[cur_node].children[0];
            let rn = self[cur_node].children[1];
            self[ln].children[1] = rn;
            self[rn].children[0] = ln;
            let item = self[cur_node].data;
            self.free(cur_node);

            if cur_node == self.front {
                self.front = rn;
            }

            if cur_node == self.rear {
                self.rear = ln;
            }

            Some(item)
        }
    }

    fn allocate(&mut self, data: T) -> u32 {
        if self.pool == !0 {
            self.memory.push(Node::default().width_data(data));
            self.memory.len() as u32 - 1
        } else {
            let old_pool = self.pool;
            let new_pool = self[old_pool].children[0];
            self[old_pool].data = data;
            self[old_pool].nullify();
            self.pool = new_pool;
            old_pool
        }
    }
}

impl<NodeType> Index<u32> for DoublyLinkedList<NodeType> {
    type Output = NodeType;
    fn index(&self, index: u32) -> &Self::Output {
        &self.memory[index as usize]
    }
}

impl<NodeType> IndexMut<u32> for DoublyLinkedList<NodeType> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.memory.get_mut(index as usize).unwrap()
    }
}
