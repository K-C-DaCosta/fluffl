use super::*;
use std::fmt::Debug;

mod iterators;
mod sort_util;
mod swappable;

use self::{iterators::*, sort_util::*, swappable::*};

/// ## Description
/// stores a tree by keeping nodes in pre-order traversal in a vector.
/// I call this a "linear" tree because this data structure only stores the parent info
/// making it more compact with `O(|V|)`. Removing edge info makes insert take`O(|V|^2)` time instead of `O(1)` time
/// so this tree structure should not be used if heavy tree manipulation is required.
/// This tree is advantagous only when it is *static*
pub struct LinearTree<T> {
    order: Vec<u32>,
    level: Vec<u32>,
    parent: Vec<Ptr>,
    data: Vec<Option<T>>,
    parent_stack: Vec<usize>,
    node_id: Vec<NodeID>,
    node_id_counter: usize,
    has_child: Vec<bool>,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct NodeID(usize);

impl<T> LinearTree<T>
where
    T: Display + Debug,
{
    pub fn new() -> Self {
        Self {
            order: vec![],
            level: vec![],
            parent: vec![],
            data: vec![],
            parent_stack: vec![],
            node_id: vec![],
            node_id_counter: 0,
            has_child: vec![],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Ptr, &T)> + '_ {
        self.data
            .iter()
            .enumerate()
            .map(|(k, i)| Some(k).zip(i.as_ref()))
            .filter_map(|a| a)
            .map(|(idx, data)| (Ptr::from(idx), data))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Ptr, &mut T)> + '_ {
        self.data
            .iter_mut()
            .enumerate()
            .map(|(k, i)| Some(k).zip(i.as_mut()))
            .filter_map(|a| a)
            .map(|(idx, data)| (Ptr::from(idx), data))
    }

    fn get_parent_ptr(&self, parent_id: NodeID) -> Ptr {
        self.node_id
            .iter()
            .enumerate()
            .find(|(_idx, &nid)| nid == parent_id)
            .map(|(idx, _)| Ptr::from(idx))
            .unwrap_or(Ptr::null())
    }

    pub fn add(&mut self, data: T, parent_id: NodeID) -> NodeID {
        let parent = self.get_parent_ptr(parent_id);
        let (nid, _) = self.allocate_node(data, parent);

        #[cfg(debug_assertions)]
        {
            if self.parent[0] != Ptr::null() || self.data[0].is_none() {
                panic!("always add parent first");
            }
        }

        self.recompute_prefix_ordering();
        self.reconstruct_parent_pointers_using_ordering_info();

        nid
    }
    fn recompute_prefix_ordering(&mut self) {
        self.parent_stack.clear();

        let root = Ptr::from(0);

        self.compute_post_order_traversal(root);

        let order = &mut self.order;
        let data = &mut self.data;
        let node_id = &mut self.node_id;
        let level = &mut self.level;

        //sort everything in post_order_traversal
        quick_co_sort(
            order,
            [
                &mut Swappable::new(data),
                &mut Swappable::new(level),
                &mut Swappable::new(node_id),
            ],
        );
    }
    fn recompute_has_child_table(&mut self) {
        let len = self.has_child.len();
        for ptr in 0..len {
            let has_children = self.get_child_nodes(ptr).next().is_some();
            self.has_child[ptr] = has_children;
        }
    }

    fn reconstruct_parent_pointers_using_ordering_info(&mut self) {
        self.recompute_has_child_table();
        self.reconstruct_parent_pointers();
    }

    fn reconstruct_parent_pointers(&mut self) {
        let root = Ptr::from(0);
        let level = &mut self.level;
        let parent_stack = &mut self.parent_stack;
        let has_child = &mut self.has_child;
        let parent = &mut self.parent;
        let node_len = level.len();

        parent_stack.clear();
        parent_stack.push(root.as_usize());
        // println!("stack-V");

        for cur_node in 1..node_len {
            // println!("stack-{:?}", self.parent_stack);

            let cur_level = level[cur_node] as usize;
            let diff = cur_level as isize - level[cur_node - 1] as isize;

            if diff < 0 {
                while parent_stack.last().is_some()
                    && level[*parent_stack.last().unwrap()] as usize != cur_level
                {
                    parent_stack.pop();
                }
                parent_stack.pop();
            }

            parent[cur_node] = Ptr::from(*parent_stack.last().expect("root should exist"));
            if has_child[cur_node] {
                parent_stack.push(cur_node);
            }
        }
    }

    pub fn print(&mut self) {
        let mut indents = String::new();

        for (signal, item) in StackSignalIterator::new(self) {
            match signal {
                StackSignal::Push => indents.push_str("->"),
                StackSignal::Pop { n_times } => (0..2 * n_times).for_each(|_| {
                    indents.pop();
                }),
                StackSignal::Nop => (),
            }
            println!("{}{}", indents, item);
        }
    }

    fn compute_post_order_traversal(&mut self, root: Ptr) {
        let mut order_idx = 0;
        self.order.iter_mut().for_each(|e| *e = !0);
        self.compute_post_order_traversal_helper(root, 0, &mut order_idx)
    }

    fn compute_post_order_traversal_helper(&mut self, root: Ptr, level: u32, order_idx: &mut u32) {
        let self_ptr = self as *mut Self;
        // let data = &mut self.data;

        let root_idx = root.as_usize();
        self.level[root_idx] = level;
        self.order[root_idx] = *order_idx;

        // println!(
        //     "[{},ord:{}]",
        //     data[root_idx].as_ref().unwrap(),
        //     self.order[root_idx]
        // );
        *order_idx += 1;

        for root in self.get_child_nodes(root) {
            let local_self = unsafe { &mut *self_ptr };
            local_self.compute_post_order_traversal_helper(root, level + 1, order_idx);
        }
    }

    fn get_child_nodes<PTR>(&self, root: PTR) -> impl Iterator<Item = Ptr> + '_
    where
        PTR: Into<Ptr>,
    {
        let root = root.into();
        let data = &self.data;
        let parent = &self.parent;
        data.iter()
            .enumerate()
            .filter_map(|(k, d)| Some(Ptr::from(k)).zip(d.as_ref()))
            .map(|(ptr, _)| ptr)
            .filter(move |ptr| parent[ptr.as_usize()] == root)
    }

    fn allocate_node(&mut self, data: T, parent: Ptr) -> (NodeID, Ptr) {
        let node_id = NodeID(self.node_id_counter);
        self.order.push(!0);
        self.data.push(Some(data));
        self.level.push(0);
        self.parent.push(parent);
        self.node_id.push(node_id);
        self.has_child.push(false);
        self.node_id_counter += 1;
        (node_id, Ptr::from(self.data.len() - 1))
    }

    pub fn remove(&mut self, node: Ptr) -> Option<(Ptr, T)> {
        unimplemented!("remove not implemented")
        // let idx = node.as_usize();
        // let parent = self.parent.get(idx).map(|&a| a);
        // if parent.is_some() {
        //     self.free_node_list.push(node);
        //     self.order[idx] = !0;
        // }
        // parent.zip(self.data[idx].take())
    }
}

#[test]
pub fn tree_test() {
    let mut tree = LinearTree::<i32>::new();
    let root = tree.add(1, NodeID::default());

    let lb = tree.add(2, root);
    let rb = tree.add(3, root);
    tree.add(5, lb);
    tree.add(4, rb);

    tree.print();
}
