use super::*;
use crate::mem;

#[derive(Copy, Clone, Debug)]
pub enum StackSignal {
    Push,
    ///pop `n`  times
    Pop {
        n_times: usize,
    },
    Nop,
}

/// Iterates through the list but also tells you how many times you should push or pop from your stack
pub struct StackSignalIterator<'a, T> {
    tree: &'a LinearTree<T>,
    covered_root: bool,
    cur_node: usize,
    node_len: usize,
}
impl<'a, T> StackSignalIterator<'a, T> {
    pub fn new(tree: &'a LinearTree<T>) -> Self {
        let len = tree.len();
        Self {
            tree,
            covered_root: false,
            cur_node: 1,
            node_len: len,
        }
    }
}
impl<'a, T> Iterator for StackSignalIterator<'a, T> {
    type Item = (StackSignal, NodeID, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.tree;

        if self.covered_root == false {
            self.covered_root = true;
            return Some((StackSignal::Nop, tree.node_id[0], unsafe {
                tree.data[0].assume_init_ref()
            }));
        }

        let level = &tree.level;
        let data = &tree.data;
        let node_len = self.node_len;
        let cur_node_ref = &mut self.cur_node;

        let cur_node = *cur_node_ref;

        (cur_node < node_len).then(move || {
            *cur_node_ref += 1;
            let cur_level = level[cur_node];
            let diff = cur_level as isize - level[cur_node - 1] as isize;

            let signal = if diff == 0 {
                StackSignal::Nop
            } else if diff > 0 {
                StackSignal::Push
            } else {
                StackSignal::Pop {
                    n_times: diff.abs() as usize,
                }
            };

            (signal, tree.node_id[cur_node], unsafe {
                data[cur_node].assume_init_ref()
            })
        })
    }
}

/// Iterates through the list but also tells you how many times you should push or pop from your stack
pub struct StackSignalIteratorMut<'a, T> {
    tree: &'a mut LinearTree<T>,
    covered_root: bool,
    cur_node: usize,
    node_len: usize,
}
impl<'a, T> StackSignalIteratorMut<'a, T> {
    pub fn new(tree: &'a mut LinearTree<T>) -> Self {
        let len = tree.len();
        Self {
            tree,
            covered_root: false,
            cur_node: 1,
            node_len: len,
        }
    }
}
impl<'a, T> Iterator for StackSignalIteratorMut<'a, T> {
    type Item = (StackSignal, NodeID, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        //allows me to split-borrow the tree
        let tree = unsafe { mem::force_borrow_mut(self.tree) };

        if self.covered_root == false {
            self.covered_root = true;
            return Some((StackSignal::Nop, tree.node_id[0], unsafe {
                tree.data[0].assume_init_mut()
            }));
        }

        let level = &mut tree.level;
        let data = &mut tree.data;
        let node_id = &mut tree.node_id;
        
        let node_len = self.node_len;
        let cur_node_ref = &mut self.cur_node;
        let cur_node = *cur_node_ref;

        (cur_node < node_len).then(move || {
            *cur_node_ref += 1;
            let cur_level = level[cur_node];
            let diff = cur_level as isize - level[cur_node - 1] as isize;

            let signal = if diff == 0 {
                StackSignal::Nop
            } else if diff > 0 {
                StackSignal::Push
            } else {
                StackSignal::Pop {
                    n_times: diff.abs() as usize,
                }
            };

            (signal, node_id[cur_node], unsafe {
                data[cur_node].assume_init_mut()
            })
        })
    }
}
