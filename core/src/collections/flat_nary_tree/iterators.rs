use tokio::signal::unix::SignalKind;

use super::*;
use crate::mem;

#[derive(Copy, Clone)]
pub enum StackSignal {
    Push,
    Pop(usize),
    Nop,
}

pub struct StackSignalIterator<'a, T> {
    tree: &'a mut LinearTree<T>,
    covered_root: bool,
    cur_node: usize,
}
impl<'a, T> StackSignalIterator<'a, T> {
    pub fn new(tree: &'a mut LinearTree<T>) -> Self {
        tree.parent_stack.clear();
        tree.parent_stack.push(0);
        Self {
            tree,
            covered_root: false,
            cur_node: 1,
        }
    }
}
impl<'a, T> Iterator for StackSignalIterator<'a, T> {
    type Item = (StackSignal, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let tree = unsafe { mem::force_borrow_mut(self.tree) };

        if self.covered_root == false {
            self.covered_root = true;
            return Some((StackSignal::Nop, tree.data[0].as_mut().unwrap()));
        }

        let level = &mut tree.level;
        let parent_stack = &mut tree.parent_stack;
        let data = &mut tree.data;
        let node_len = level.len();
        let cur_node_ref = &mut self.cur_node;

        let cur_node = *cur_node_ref;
        // let cur_node_ptr = Ptr::from(cur_node);

        (cur_node < node_len).then(move || {
            *cur_node_ref += 1;

            let cur_level = level[cur_node];
            let diff = cur_level as isize - level[cur_node - 1] as isize;
            let mut pop_count = 0; 

            if diff < 0 {
                while parent_stack.last().is_some()
                    && level[*parent_stack.last().unwrap()] != cur_level
                {
                    parent_stack.pop();
                    pop_count+=1;
                }
            }

            let signal = if diff > 0 {
                parent_stack.push(cur_node);
                StackSignal::Push
            } else if diff < 0 {
                StackSignal::Pop(pop_count)
            } else {
                StackSignal::Nop
            };

            (signal, data[cur_node].as_mut().unwrap())
        })
    }
}
