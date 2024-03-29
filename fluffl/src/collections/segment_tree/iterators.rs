#![allow(dead_code)]
pub use super::{super::Ptr, *};

pub struct ScalarSearchIter<'a, V> {
    tree: &'a CircularSegmentTree<V>,
    node: Ptr,
    node_interval: Interval,
    t: FP64,
}
impl<'a, V> ScalarSearchIter<'a, V> {
    pub fn new(tree: &'a CircularSegmentTree<V>, t: FP64) -> Self {
        Self {
            tree,
            node: tree.linear_tree.root(),
            node_interval: Interval::from((0, tree.width)),
            t,
        }
    }
}

impl<'a, V> Iterator for ScalarSearchIter<'a, V> {
    type Item = Ptr;
    fn next(&mut self) -> Option<Self::Item> {
        let exponent = self.tree.exponent;
        let node = self.node;
        let t = self.t;
        let node_interval = self.node_interval;
        let child_intervals = [node_interval.chunk(2, 0), node_interval.chunk(2, 1)];
        let wrapped_t = t.fast_mod(exponent as u8);

        // if node != Ptr::null(){
        //     println!("t = {} int =>{:?} vals:{:?}",t & remainder_mask, node_interval, self.tree.bucket_pool[ self.tree.linear_tree[node].data.unwrap()] );
        // }

        (node != Ptr::null()).then(|| {
            let node_info = &self.tree.linear_tree[node];

            if child_intervals[0].is_within(wrapped_t) {
                self.node = node_info.children[0];
                self.node_interval = child_intervals[0];
            } else {
                self.node = node_info.children[1];
                self.node_interval = child_intervals[1];
            }

            node
        })
    }
}

#[derive(Copy, Clone, Default)]
pub struct SearchIterFrame {
    root: Ptr,
    child_cursor: usize,
    interval: Interval,
    visited: bool,
}

/// Used to do a DFS
pub struct IntervalSearchIter<'a, V> {
    tree: &'a CircularSegmentTree<V>,
    stack: FixedStack<64, SearchIterFrame>,
    query_interval: Interval,
}
impl<'a, V> IntervalSearchIter<'a, V> {
    pub fn new(tree: &'a CircularSegmentTree<V>, query_interval: Interval) -> Self {
        let mut stack = FixedStack::new();
        stack.push(SearchIterFrame {
            root: tree.linear_tree.root(),
            child_cursor: 0,
            visited: false,
            interval: Interval::from((0, tree.width)),
        });
        Self {
            tree,
            stack,
            query_interval,
        }
    }
}
impl<'a, V> Iterator for IntervalSearchIter<'a, V> {
    type Item = Option<Ptr>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|mut sf| {
            let root = sf.root;
            let cursor = sf.child_cursor;
            let interval = sf.interval;

            let root_has_no_entries = self.tree.linear_tree[root]
                .data
                .map(|bucket_idx| self.tree.bucket_pool[bucket_idx].is_empty())
                .unwrap_or_default();

            let yield_res = (!sf.visited && !root_has_no_entries).then_some(root);
            sf.visited = true;
            sf.child_cursor += 1;

            if cursor < 2 {
                self.stack.push(sf);
                let child = self.tree.linear_tree[root].children[cursor];
                let child_interval = interval.chunk(2, cursor);

                if child != Ptr::null() && child_interval.is_overlapping(&self.query_interval) {
                    self.stack.push(SearchIterFrame {
                        root: child,
                        child_cursor: 0,
                        visited: false,
                        interval: child_interval,
                    });
                }
            }
            yield_res
        })
    }
}
