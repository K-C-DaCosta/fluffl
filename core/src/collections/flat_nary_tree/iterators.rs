use tokio::signal::unix::SignalKind;

use super::*;
use crate::mem;

#[derive(Copy, Clone)]
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
    tree: &'a mut LinearTree<T>,
    covered_root: bool,
    cur_node: usize,
    node_len: usize,
}
impl<'a, T> StackSignalIterator<'a, T>
where
    T: Display + Debug,
{
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
impl<'a, T> Iterator for StackSignalIterator<'a, T> {
    type Item = (StackSignal, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        //allows me to split-borrow the tree
        let tree = unsafe { mem::force_borrow_mut(self.tree) };

        if self.covered_root == false {
            self.covered_root = true;
            return Some((StackSignal::Nop, tree.data[0].as_mut().unwrap()));
        }

        let level = &mut tree.level;
        let data = &mut tree.data;
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

            (signal, data[cur_node].as_mut().unwrap())
        })
    }
}
