use super::*;

use std::{fmt::Debug, mem::MaybeUninit};
mod iterators;
pub mod sort_util;
pub mod swappable;

pub use self::iterators::StackSignal;
use self::{iterators::*, sort_util::*, swappable::*};

#[derive(Copy, Clone, Hash, Default, Debug, PartialEq, Eq)]
pub struct NodeID(pub usize);
impl NodeID {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct NodeInfo<'a, T> {
    pub parent: Option<NodeID>,
    pub id: NodeID,
    pub val: &'a T,
}

pub struct NodeInfoMut<'a, T> {
    pub parent: Option<NodeID>,
    pub id: NodeID,
    pub val: &'a mut T,
}

/// ## Description
/// stores a tree by keeping nodes in **pre-order** traversal in a vector for fast traversal.
/// I call this a "linear" tree because this data structure only stores the parent info
/// making it more compact with `O(|V|)` space. Removing edge info makes insert take`O(|V|^2)` time instead of `O(1)` time.
/// so this tree structure should not be used if heavy tree manipulation is required.
/// This tree is advantagous only when it is *static* , meaning tree topology isnt changing
pub struct LinearTree<T> {
    order: Vec<u32>,
    level: Vec<u32>,
    parent: Vec<Ptr>,
    data: Vec<MaybeUninit<T>>,
    parent_stack: Vec<usize>,
    node_id: Vec<NodeID>,
    node_id_counter: usize,
    /// after co-sorting, deleted nodes bubble to the end of the list for reusing data
    nodes_deleted: usize,
    id_to_ptr_table: Vec<Ptr>,
}

impl<T> LinearTree<T> {
    pub fn new() -> Self {
        Self {
            order: vec![],
            level: vec![],
            parent: vec![],
            data: vec![],
            parent_stack: Vec::with_capacity(128),
            node_id: vec![],
            node_id_counter: 0,
            nodes_deleted: 0,
            id_to_ptr_table: Vec::new(),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, len) }
    }

    /// ## Description
    /// assign `new_parent_id` to the parent pointer of `id`   
    /// ## Comments
    /// - Complexity is O(1)
    pub fn set_parent<NID: Copy + Into<NodeID>>(&mut self, id: NID, new_parent_id: NID) {
        let current_ptr = self.resolve_id_to_ptr(id.into());
        let new_parent_ptr = self.resolve_id_to_ptr(new_parent_id.into());
        self.parent[current_ptr.as_usize()] = new_parent_ptr;
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = NodeInfo<'a, T>> {
        let len = self.len();

        let data = &self.data;
        let parent = &self.parent;
        let node_id = &self.node_id;

        data.iter()
            .zip(parent.iter())
            .enumerate()
            .take(len)
            .map(|(cur_ptr_usize, (val, &parent_ptr))| unsafe {
                (cur_ptr_usize, parent_ptr, val.assume_init_ref())
            })
            .map(move |(cur_ptr_usize, parent_ptr, val)| NodeInfo {
                val,
                id: node_id[cur_ptr_usize],
                parent: (parent_ptr != Ptr::null()).then(|| node_id[parent_ptr.as_usize()]),
            })
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = NodeInfoMut<'a, T>> {
        let len = self.len();
        let data = &mut self.data;
        let parent = &mut self.parent;
        let node_id = &mut self.node_id;

        data.iter_mut()
            .zip(parent.iter_mut())
            .enumerate()
            .take(len)
            .map(|(cur_ptr_usize, (val, &mut parent_ptr))| unsafe {
                (cur_ptr_usize, parent_ptr, val.assume_init_mut())
            })
            .map(move |(cur_ptr_usize, parent_ptr, val)| NodeInfoMut {
                val,
                id: node_id[cur_ptr_usize],
                parent: (parent_ptr != Ptr::null()).then(|| node_id[parent_ptr.as_usize()]),
            })
    }

    pub fn get<NID>(&self, node_id: NID) -> Option<&T>
    where
        NID: Copy + Into<NodeID>,
    {
        let node_ptr = self.resolve_id_to_ptr(node_id.into());
        if node_ptr == Ptr::null() {
            return None;
        }
        self.data
            .get(node_ptr.as_usize())
            .map(|val| unsafe { val.assume_init_ref() })
    }

    pub fn get_mut<NID>(&mut self, node_id: NID) -> Option<&mut T>
    where
        NID: Copy + Into<NodeID>,
    {
        let node_ptr = self.resolve_id_to_ptr(node_id.into());
        self.data
            .get_mut(node_ptr.as_usize())
            .map(|val| unsafe { val.assume_init_mut() })
    }

    /// get the underlying
    pub fn get_mut_uninit<NID>(&mut self, node_id: NID) -> &mut MaybeUninit<T>
    where
        NID: Copy + Into<NodeID>,
    {
        let node_ptr = self.resolve_id_to_ptr(node_id.into());
        self.data
            .get_mut(node_ptr.as_usize())
            .expect("node_id invalid")
    }

    pub fn get_parent_id<NID>(&self, id: NID) -> Option<NodeID>
    where
        NID: Copy + Into<NodeID>,
    {
        let node_ptr = self.resolve_id_to_ptr(id.into());
        let parent_ptr = self.parent[node_ptr.as_usize()];
        (parent_ptr != Ptr::null()).then(|| self.node_id[parent_ptr.as_usize()])
    }

    fn resolve_id_to_ptr(&self, id: NodeID) -> Ptr {
        self.id_to_ptr_table
            .get(id.as_usize())
            .map(|&a| a)
            .unwrap_or(Ptr::null())
    }

    /// Partially adds nodes to tree but tree isn't valid until
    /// `Self::reconstruct_preorder(..)` is called
    /// ## Comments
    /// -  O(1) Complexity. This is ,uch faster than `Self::add(..)`
    /// - the node attributes like `parent` of newly added nodes are safe to mutate using this method
    pub unsafe fn add_deffered_reconstruction(
        &mut self,
        data: MaybeUninit<T>,
        parent_id: NodeID,
    ) -> NodeID {
        let parent = self.resolve_id_to_ptr(parent_id);
        let (nid, _) = self.allocate_node_uninit(data, parent);
        #[cfg(debug_assertions)]
        {
            if self.parent[0] != Ptr::null() {
                panic!("always add parent first");
            }
        }
        nid
    }

    pub fn add(&mut self, data: T, parent_id: NodeID) -> NodeID {
        let parent = self.resolve_id_to_ptr(parent_id);
        let (nid, _) = self.allocate_node(data, parent);

        #[cfg(debug_assertions)]
        {
            if self.parent[0] != Ptr::null() {
                panic!("always add parent first");
            }
        }

        self.reconstruct_preorder();

        nid
    }
    fn recompute_prefix_ordering(&mut self) {
        self.parent_stack.clear();

        let root = Ptr::from(0);

        self.compute_pre_order_traversal(root);

        let order = &mut self.order;
        let data = &mut self.data;
        let node_id = &mut self.node_id;
        let level = &mut self.level;

        // sort everything in pre_order_traversal
        quick_co_sort(
            order,
            [
                &mut Swappable::new(data),
                &mut Swappable::new(level),
                &mut Swappable::new(node_id),
            ],
        );
    }

    /// ## Description
    /// Sorts nodes in preorder for fast traversal
    /// - each element in array corresponds to a node attribute
    /// - DFSes the tree in post-order using the array `order` to label the vertex order
    ///     - also computes node `level` in the DFS traversal
    /// - sort by `order` but also co-sort arrays associated with vertex
    /// - once the all the lists are sorted in post-order I use `level` to recompute parent pointers
    /// ## Complexity
    /// `O(|V|log(|V|))`
    /// ## Comments
    /// - index 0 in all the node attribute arrays ALWAYS means the root of the tree
    pub fn reconstruct_preorder(&mut self) {
        self.recompute_prefix_ordering();
        self.reconstruct_parent_pointers();

        // after the above functions are called,
        // all (nid,ptr) pairs are invalid and must be recomputed
        self.reconstruct_id_to_ptr_table();
    }

    fn reconstruct_id_to_ptr_table(&mut self) {
        for k in 0..self.data.len() {
            let nid = self.node_id[k];
            let ptr = Ptr::from(k);
            self.id_to_ptr_table[nid.as_usize()] = ptr;
        }
    }

    fn reconstruct_parent_pointers(&mut self) {
        let root = Ptr::from(0);
        let valid_nodes_len = self.len();
        let level = &mut self.level;
        let parent_stack = &mut self.parent_stack;
        let parent = &mut self.parent;

        parent_stack.clear();
        parent_stack.push(root.as_usize());

        for cur_node in 1..valid_nodes_len {
            let cur_level = level[cur_node] as usize;
            let diff = cur_level as isize - level[cur_node - 1] as isize;
            //this was the fix
            if diff <= 0 {
                while parent_stack.last().is_some()
                    && level[*parent_stack.last().unwrap()] as usize != cur_level
                {
                    parent_stack.pop();
                }
                parent_stack.pop();
            }
            parent[cur_node] = Ptr::from(*parent_stack.last().expect("root should exist"));
            parent_stack.push(cur_node);
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.nodes_deleted
    }

    fn compute_pre_order_traversal(&mut self, root: Ptr) {
        let mut order_idx = 0;
        self.order.iter_mut().for_each(|e| *e = !0);
        self.compute_pre_order_traversal_helper(root, 0, &mut order_idx)
    }

    fn compute_pre_order_traversal_helper(&mut self, root: Ptr, level: u32, order_idx: &mut u32) {
        let self_ptr = self as *mut Self;
        let root_idx = root.as_usize();
        self.level[root_idx] = level;
        let is_root_or_non_root_but_has_parent =
            root_idx == 0 || root_idx > 0 && self.parent[root_idx] != Ptr::null();

        if is_root_or_non_root_but_has_parent {
            self.order[root_idx] = *order_idx;
        }

        *order_idx += 1;

        for child in self.get_child_nodes(root) {
            let local_self = unsafe { &mut *self_ptr };
            local_self.compute_pre_order_traversal_helper(child, level + 1, order_idx);
        }
    }

    fn get_child_nodes<PTR>(&self, root: PTR) -> impl Iterator<Item = Ptr> + '_
    where
        PTR: Into<Ptr>,
    {
        let num_active_nodes = self.len();
        let root = root.into();
        let parent = &self.parent;
        (0..num_active_nodes)
            .map(|ptr| Ptr::from(ptr))
            .filter(move |ptr| parent[ptr.as_usize()] == root)
    }

    fn allocate_node(&mut self, data: T, parent: Ptr) -> (NodeID, Ptr) {
        unsafe { self.allocate_node_uninit(MaybeUninit::new(data), parent) }
    }

    unsafe fn allocate_node_uninit(&mut self, data: MaybeUninit<T>, parent: Ptr) -> (NodeID, Ptr) {
        debug_assert_eq!(
            self.nodes_deleted <= self.data.len(),
            true,
            "nodes_deleted cannot be greater than the length of the array"
        );

        if self.nodes_deleted > 0 {
            let ptr = self.data.len() - self.nodes_deleted;
            let node_id = self.node_id[ptr];
            //set new data
            self.order[ptr] = !0;
            self.data[ptr] = data;
            self.parent[ptr] = parent;
            self.level[ptr] = 0;
            //decrement nodes deleted
            self.nodes_deleted -= 1;
            (node_id, Ptr::from(ptr))
        } else {
            let node_id = NodeID(self.node_id_counter);
            //instantiate node object
            self.order.push(!0);
            self.data.push(data);
            self.level.push(0);
            self.parent.push(parent);
            self.node_id.push(node_id);
            self.id_to_ptr_table.push(Ptr::from(self.data.len() - 1));
            self.node_id_counter += 1;
            (node_id, Ptr::from(self.data.len() - 1))
        }
    }

    pub fn remove(&mut self, id: NodeID, removed_vals: &mut Vec<T>) {
        //before anything happens make sure result buffer is clear
        removed_vals.clear();

        let ptr = self.resolve_id_to_ptr(id);
        let deleted_level = self.level[ptr];
        let len = self.len();

        //remove root node
        self.remove_single_node(ptr, removed_vals);

        //remove subtree nodes
        let mut subtree_node = ptr + 1;
        while subtree_node.as_usize() < len && self.level[subtree_node] > deleted_level {
            self.remove_single_node(subtree_node, removed_vals);
            subtree_node += 1;
        }

        self.reconstruct_preorder();
    }

    fn remove_single_node(&mut self, ptr: Ptr, removed_vals: &mut Vec<T>) {
        self.order[ptr] = !0;
        self.parent[ptr] = Ptr::null();
        self.level[ptr] = !0;

        //shallow copy item
        let removed_item = unsafe { self.data[ptr].assume_init_read() };
        removed_vals.push(removed_item);

        self.data[ptr] = MaybeUninit::zeroed();

        self.nodes_deleted += 1;
    }

    pub fn iter_mut_stack_signals(&mut self) -> StackSignalIteratorMut<'_, T> {
        StackSignalIteratorMut::new(self)
    }

    pub fn iter_stack_signals(&self) -> StackSignalIterator<'_, T> {
        StackSignalIterator::new(self)
    }

    pub fn print_by_ids(&mut self) {
        let mut indents = String::new();
        let indent = "--";

        for (signal, item, _) in StackSignalIteratorMut::new(self) {
            match signal {
                StackSignal::Push => indents.push_str(indent),
                StackSignal::Pop { n_times } => (0..indent.len() * n_times).for_each(|_| {
                    indents.pop();
                }),
                StackSignal::Nop => {}
            }
            if indents.len() > 0 {
                indents.pop();
                indents.push('>');
                println!("{}{}", indents, item.as_usize());
                indents.pop();
                indents.push('-');
            } else {
                println!("{}{}", indents, item.as_usize());
            }
        }
    }
}

impl<T> LinearTree<T>
where
    T: Debug + Display,
{
    pub fn print(&mut self) {
        let mut indents = String::new();
        let indent = "--";

        for (signal, _, item) in StackSignalIteratorMut::new(self) {
            match signal {
                StackSignal::Push => indents.push_str(indent),
                StackSignal::Pop { n_times } => (0..indent.len() * n_times).for_each(|_| {
                    indents.pop();
                }),
                StackSignal::Nop => {}
            }
            if indents.len() > 0 {
                indents.pop();
                indents.push('>');
                println!("{}{}", indents, item);
                indents.pop();
                indents.push('-');
            } else {
                println!("{}{}", indents, item);
            }
        }
    }
}

#[test]
pub fn remove_sanity() {
    let mut removed_nodes = vec![];

    let mut tree = LinearTree::<i32>::new();
    let root = tree.add(1, NodeID::default());

    let lb = tree.add(2, root);
    let rb = tree.add(3, root);
    tree.add(5, lb);
    tree.add(4, rb);
    tree.add(7, rb);
    tree.add(9, rb);

    tree.print();

    for _ in 0..10 {
        tree.remove(rb, &mut removed_nodes);
        let rb = tree.add(3, root);
        tree.add(4, rb);
        tree.add(7, rb);
        tree.add(9, rb);
        // println!("{:?}",removed_nodes);
    }
    tree.print();
}

#[test]
pub fn drop_sanity() {
    use std::{cell::*, rc::*};

    let has_been_dropped = Rc::new(Cell::new(false));

    struct HasHeapStuff {
        dropped: Rc<Cell<bool>>,
        _a: String,
        _b: Vec<i32>,
    }

    impl Drop for HasHeapStuff {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }

    let droppable_item = HasHeapStuff {
        dropped: has_been_dropped.clone(),
        _a: String::from("hello world my name is adam poo poo head"),
        _b: vec![0; 1_000],
    };

    let mut removed_nodes = vec![];
    let mut tree = LinearTree::new();
    let node = tree.add(droppable_item, NodeID::default());

    //node has been removed but removed object shouldn't be dropped
    tree.remove(node, &mut removed_nodes);
    assert_eq!(false, has_been_dropped.get());

    //the clear should invoke drop
    removed_nodes.clear();
    assert_eq!(true, has_been_dropped.get());
}
