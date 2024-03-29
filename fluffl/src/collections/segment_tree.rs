#![allow(dead_code)]

use crate::{
    audio::interval::*,
    collections::{binary_tree::BinaryTree, fixed_stack::FixedStack, Ptr},
    console::*,
    console_log,
    iterators::GroupIterator,
    math::FP64,
    mem::force_static_mut,
};
use std::{
    fmt::Debug,
    ops::{Deref, Index, IndexMut},
};

pub mod index_types;
mod iterators;
mod pools;
mod tests;

use index_types::*;
use iterators::*;
use pools::*;

#[derive(Default)]
pub struct TreeIterState {
    pub intervals: [Interval; 2],
    pub aux_intervals: [Interval; 2],
}
impl TreeIterState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A Special segment tree where root is segment: \[0,`width` \]
/// all segments are interpreted to be circular
pub struct CircularSegmentTree<V> {
    max_height: u32,
    /// `width` = 2^`exponent`
    width: u64,
    /// `exponent` is between 0-127
    exponent: u64,
    linear_tree: BinaryTree<BucketIndex>,
    global_pool: GlobalIntervalPool<V>,
    bucket_pool: BucketPool,
}

impl<V: 'static> CircularSegmentTree<V> {
    pub fn new(max_height: u32, width: u64) -> Self {
        assert_eq!(width.count_ones(), 1, "width needs to be a power of two");

        let mut bucket_pool = BucketPool::new();
        let mut linear_tree = BinaryTree::new();
        let root = linear_tree.allocate(Ptr::null(), bucket_pool.allocate());
        //make sure to root always exists
        linear_tree.set_root(root);

        Self {
            max_height,
            width,
            exponent: (width - 1).count_ones() as u64,
            linear_tree,
            bucket_pool,
            global_pool: GlobalIntervalPool::new(),
        }
    }

    pub fn search_scalar(
        &self,
        time: FP64,
    ) -> impl Iterator<Item = (GlobalIndex, &'_ GlobalInterval<V>)> {
        let tree = self;
        let exponent = self.exponent as u8;
        self.bucket_search_scalar(time)
            // .map(move |ptr| {
            //     let i = compute_interval(ptr, self.width);
            //     println!("{} => {:?}", ptr, i);
            //     ptr
            // })
            .flat_map(move |ptr| tree.linear_tree[ptr].data)
            .flat_map(move |bucket_idx| tree.bucket_pool[bucket_idx].iter())
            .filter(move |interval| interval.is_within(time.fast_mod(exponent)))
            .filter_map(move |interval| {
                Some(interval.global_idx).zip(tree.global_pool[interval.global_idx].as_ref())
            })
            .filter(move |(_gidx, gi)| gi.is_within(time))
    }

    /// searches buckets that intersect `t`, where `t` is not assumed to be circular
    /// but gets converted to circular t internally
    /// ## Comments
    /// - For whatever reason, scalar searches are fastest kind of iterator
    fn bucket_search_scalar(&self, t: FP64) -> ScalarSearchIter<'_, V> {
        ScalarSearchIter::new(self, t)
    }

    pub fn search_interval<'a, 'b>(
        &'a self,
        state: &'b mut TreeIterState,
        unclipped_interval: Interval,
    ) -> impl Iterator<Item = (GlobalIndex, &'a GlobalInterval<V>)> + 'b
    where
        'a: 'b,
    {
        let clipped_intervals = &mut state.aux_intervals;
        let num_clips = self.clip_interval(unclipped_interval, clipped_intervals);
        let clipped_intervals = &clipped_intervals[0..num_clips];

        self.bucket_interval_search(unclipped_interval, &mut state.intervals)
            .flat_map(move |bucket_ptr| {
                let bucket_idx = self.linear_tree[bucket_ptr]
                    .data
                    .expect("bucket assumed to exist");

                //for each tree interval I want to check the clipped intervals
                self.bucket_pool[bucket_idx]
                    .iter()
                    .flat_map(move |&tree_interval| {
                        clipped_intervals
                            .iter()
                            .map(move |&clipped_query_interval| {
                                (tree_interval, clipped_query_interval)
                            })
                    })
            })
            .filter(move |(tree_interval, clipped_query_interval)| {
                tree_interval.is_overlapping(clipped_query_interval)
            })
            .map(move |(tree_interval, _)| {
                let global_idx = tree_interval.global_idx;
                (
                    global_idx,
                    self.global_pool[global_idx]
                        .as_ref()
                        .expect("global interval should exist"),
                )
            })
            .filter(move |&(_global_interval_idx, global_interval)| {
                global_interval.is_overlapping(&unclipped_interval)
            })
    }

    ///returns all buckets that overlap `interval`
    fn bucket_interval_search<'a, 'b>(
        &'a self,
        interval: Interval,
        clipped_intervals: &'b mut [Interval; 2],
    ) -> impl Iterator<Item = Ptr> + 'b
    where
        'a: 'b,
    {
        let num_clipped = self.clip_interval(interval, clipped_intervals);
        clipped_intervals[0..num_clipped]
            .iter()
            .flat_map(move |&i| IntervalSearchIter::new(self, i).flatten())
    }

    /// ## Description
    /// inserts interval and value associated with it
    /// ## Returns
    /// an Index referencing the (interval,data) pair, which you can then use to modify the data while its in the tree
    pub fn insert(&mut self, interval: Interval, data: V) -> GlobalIndex {
        let alias = self.global_pool.allocate(GlobalInterval { interval, data });

        //clip the intervals and make them circular
        let mut clipping_results = [Interval::default(); 2];
        let num_clips = self.clip_interval(interval, &mut clipping_results);

        for &interval in &clipping_results[0..num_clips] {
            self.insert_helper(interval, alias, self.linear_tree.root(), 0, 0, self.width);
        }

        alias
    }

    fn insert_helper(
        &mut self,
        interval: Interval,
        alias: GlobalIndex,
        mut root: Ptr,
        mut height: u32,
        mut lo: u64,
        mut hi: u64,
    ) {
        let bucket_pool = &mut self.bucket_pool;
        let linear_tree = &mut self.linear_tree;

        let tree_interval = TreeInterval {
            clipped_interval: interval,
            global_idx: alias,
        };

        while height < self.max_height {
            let mid = lo + (hi - lo) / 2;
            let overlapping_left = interval.is_overlapping(&Interval::from((lo, mid)));
            let overlapping_right = interval.is_overlapping(&Interval::from((mid, hi)));
            if overlapping_left != overlapping_right {
                //select root
                let selected_subtree = if overlapping_left {
                    hi = mid;
                    0
                } else {
                    lo = mid;
                    1
                };

                //if selected subtree is null allocate a node for it
                if linear_tree[root].children[selected_subtree] == Ptr::null() {
                    let new_bucket = bucket_pool.allocate();
                    let new_node = linear_tree.allocate(root, new_bucket);
                    linear_tree[root].children[selected_subtree] = new_node;
                }

                root = linear_tree[root].children[selected_subtree];

                //increment depth counter
                height += 1;
            } else {
                //cant go any further without splitting
                //so stop and insert
                break;
            }
        }

        //insert
        let bucket_idx = linear_tree[root].data.unwrap();
        bucket_pool[bucket_idx].push(tree_interval);
    }

    /// ## Description
    /// removes the value associated with the interval from the tree given an `idx`
    /// ### Comments
    /// - works by binary searching the tree of buckets looking for anything that points to 'gid' and removing it
    /// - its an O(H) = O(log(N)) operation, where  N = 2^H, and H is the height of the tree   
    pub fn remove_by_global_idx(&mut self, gid: GlobalIndex) -> Option<V> {
        // this is safe to do, but breaks rust aliasing rules so
        let tree_ref = unsafe { force_static_mut(self) };

        //fetch global interval
        let global_interval = self.global_pool[gid]
            .as_ref()
            .expect("gidx should exist")
            .interval;
        //compute clipped intervals from global
        let mut clipped_intervals = [Interval::default(); 2];
        let num_clips = self.clip_interval(global_interval, &mut clipped_intervals);

        //binary search through buckets and remove intervals that alias gid
        clipped_intervals[0..num_clips]
            .iter()
            .flat_map(|clipped_interval| {
                //use the midpoint of the clipped intervals as a lookup into the tree
                let midpoint = clipped_interval.midpoint();
                self.bucket_search_scalar(midpoint)
            })
            .for_each(|bucket_ptr| {
                tree_ref
                    .remove_item_from_bucket_by_predicate(bucket_ptr, |(_, clipped_interval)| {
                        clipped_interval.global_idx == gid
                    });
            });

        self.global_pool.free(gid).map(|interval| interval.data)
    }

    fn remove_by_global_idx_helper(&mut self, root: Ptr, idx: GlobalIndex) {
        if root == Ptr::null() {
            return;
        }

        self.remove_item_from_bucket_by_predicate(root, |(_, clipped_interval)| {
            clipped_interval.global_idx == idx
        });

        for k in 0..2 {
            let child = self.linear_tree[root].children[k];
            self.remove_by_global_idx_helper(child, idx);
        }
    }

    /// ## Description
    /// Removes **ALL** values, `V`, that *exactly* match that `query_interval`
    /// ### Returns
    /// An iterator that when executed will remove an item that matches `query_interval`
    pub fn remove<'a, 'b>(
        &'a mut self,
        state: &'b mut TreeIterState,
        query_interval: Interval,
    ) -> impl Iterator<Item = GlobalInterval<V>> + 'b
    where
        'a: 'b,
    {
        let clip_results = &mut state.intervals;
        let num_clips = self.clip_interval(query_interval, clip_results);

        let seg_tree_ptr = self as *mut CircularSegmentTree<V> as *const CircularSegmentTree<V>;
        let read_only_self = unsafe { &*seg_tree_ptr };

        let nodes_to_delete = clip_results[0..num_clips]
            .iter()
            .map(|&interval| interval.midpoint())
            .flat_map(move |midpoint| {
                read_only_self
                    .bucket_search_scalar(midpoint)
                    .filter_map(move |ptr| Some(ptr).zip(read_only_self.linear_tree[ptr].data))
                    .map(move |(ptr, bucket_index)| {
                        (ptr, &read_only_self.bucket_pool[bucket_index], midpoint)
                    })
            })
            .flat_map(move |(ptr, interval_list, midpoint)| {
                interval_list
                    .iter()
                    // have to traverse from END to START otherwise deleting multiple entries in the bucket becomes problematic
                    .rev()
                    .map(move |&interval| (ptr, interval, midpoint))
            })
            .filter(move |&(_ptr, tree_interval, midpoint)| {
                tree_interval.is_within(midpoint)
                    && self.global_pool[tree_interval.global_idx]
                        .as_ref()
                        .expect("global_idx should be valid here")
                        .interval
                        == query_interval
            });

        GroupIterator::new(
            nodes_to_delete,
            move |(_bucket_ptr, b, _)| b.global_idx,
            move |&(ptr, tree_interval, _)| {
                let mut_self = unsafe {
                    &mut *(seg_tree_ptr as *const CircularSegmentTree<V>
                        as *mut CircularSegmentTree<V>)
                };
                // println!(
                //     "remove tree interval {:?} at bucket node: {:?}",
                //     tree_interval, ptr
                // );
                mut_self.remove_helper(ptr, tree_interval);
            },
        )
        .flatten()
        .map(move |(_bucket_ptr, tree_interval, _)| {
            let mut_self = unsafe {
                &mut *(seg_tree_ptr as *const CircularSegmentTree<V> as *mut CircularSegmentTree<V>)
            };
            mut_self
                .global_pool
                .free(tree_interval.global_idx)
                .expect("value should be here")
        })
    }

    fn remove_helper(&mut self, root: Ptr, tree_interval: TreeInterval) {
        // traversing the tree  happens in the remove iterator
        //simply remove
        self.remove_item_from_bucket_by_predicate(root, |(_, e)| {
            e.clipped_interval == tree_interval.clipped_interval
        });
    }

    /// ## description
    /// removes clipped intervals from a bucket. If nothing matches the predicate, nothing will happen
    /// ### comments
    /// this function **DOES NOT** traverse the tree, it simply removes clipped intervals from the bucket at `root`
    fn remove_item_from_bucket_by_predicate<PRED>(&mut self, mut root: Ptr, predicate: PRED)
    where
        PRED: FnMut(&(usize, &TreeInterval)) -> bool,
    {
        let global_root = self.linear_tree.root();
        let mut bucket_idx = self.linear_tree[root]
            .data
            .expect("Bucket index should always exist");

        //search for the tree interval index in the bucket in reverse
        let tree_interval_index_opt = self.bucket_pool[bucket_idx]
            .iter()
            .enumerate()
            .rev()
            .find(predicate)
            .map(|(i, _)| i);

        if let Some(tree_interval_index) = tree_interval_index_opt {
            // method 2 - swap and pop:
            // maybe this fixes the problem? edit( yup fixes it, because order in the bucket list not important to me
            // also avoids shifting which keeps vec::iter() happy

            self.bucket_pool[bucket_idx].swap_remove(tree_interval_index);

            while root != Ptr::null()
                && root != global_root
                && self.bucket_pool[bucket_idx].is_empty()
                && self.linear_tree[root].is_leaf()
            {
                let parent = self.linear_tree[root].parent;
                bucket_idx = self.linear_tree[root].data.unwrap();

                //remove node from parent
                if parent != Ptr::null() {
                    if self.linear_tree[parent].children[0] == root {
                        self.linear_tree[parent].children[0] = Ptr::null();
                    } else {
                        self.linear_tree[parent].children[1] = Ptr::null();
                    }
                }

                self.linear_tree.free(root);
                self.bucket_pool.free(bucket_idx);
                root = parent;

                //update bucket index
                bucket_idx = self.linear_tree[root].data.unwrap();
            }
        }
    }

    /// returns number of clips
    fn clip_interval(&self, interval: Interval, clippings: &mut [Interval; 2]) -> usize {
        let exponent = self.exponent as u8;
        let Interval { lo, hi } = interval;

        let zero = FP64::from(0);
        let width = FP64::from(self.width);

        // println!("interval = {:?}",interval);

        let lo_block = lo >> exponent;
        let hi_block = hi >> exponent;
        let num_blocks_interval_spans = (hi_block.floor() - lo_block.floor()) + FP64::from(1);

        let split_a = Interval::from((lo.fast_mod(exponent), width));
        let split_b = Interval::from((zero, hi.fast_mod(exponent)));
        let splic_c = Interval::from((lo.fast_mod(exponent), hi.fast_mod(exponent)));

        //clip the intervals and make them circular
        if num_blocks_interval_spans >= FP64::from(3) {
            //this case the interval spans multiple blocks so insert it at the root and stop there
            clippings[0] = Interval::from((zero, width));
            1
        } else if num_blocks_interval_spans >= FP64::from(2)
            && split_a.distance() > FP64::zero()
            && split_b.distance() > FP64::zero()
        {
            //this interval spans two blocks so it can be broken up more efficiently
            clippings[0] = split_a;
            clippings[1] = split_b;
            2
        } else if !splic_c.inverted() {
            //interval doesn't sit on a block boundary so clipping is not needed
            clippings[0] = splic_c;
            1
        } else {
            clippings[0] = interval;
            1
        }
    }

    pub fn print_tree(&self, space: &str) {
        self.print_tree_helper(self.linear_tree.root(), &mut String::new(), space)
    }

    fn print_tree_helper(&self, root: Ptr, space_stack: &mut String, space: &str) {
        if root == Ptr::null() {
            return;
        }

        space_stack.push_str(space);
        self.print_tree_helper(self.linear_tree[root].children[1], space_stack, space);
        for _ in 0..space.len() {
            space_stack.pop();
        }

        let bi = self.linear_tree[root].data.unwrap();
        console_log!(
            "{}{:?}\n",
            space_stack,
            self.bucket_pool[bi]
                .iter()
                .map(|ti| ti.clipped_interval)
                .collect::<Vec<_>>()
        );

        space_stack.push_str(space);
        self.print_tree_helper(self.linear_tree[root].children[0], space_stack, space);
        for _ in 0..space.len() {
            space_stack.pop();
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.global_pool
            .iter()
            .filter_map(|item| item.as_ref())
            .map(|item| &item.data)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.global_pool
            .iter_mut()
            .filter_map(|item| item.as_mut())
            .map(|item| &mut item.data)
    }
}

impl<V> Index<GlobalIndex> for CircularSegmentTree<V> {
    type Output = V;
    fn index(&self, index: GlobalIndex) -> &Self::Output {
        &self.global_pool[index]
            .as_ref()
            .expect("global index is invalid")
            .data
    }
}

impl<V> IndexMut<GlobalIndex> for CircularSegmentTree<V> {
    fn index_mut(&mut self, index: GlobalIndex) -> &mut Self::Output {
        &mut self.global_pool[index]
            .as_mut()
            .expect("global index is invalid")
            .data
    }
}

pub fn rand_lehmer64(state: &mut u128) -> u64 {
    *state = state.wrapping_mul(0xda942042e4dd58b5);
    (*state >> 64) as u64
}

#[test]
fn clip_interval_bug() {
    let tree = CircularSegmentTree::<u32>::new(4, 1024);
    let mut clippings = [Interval::default(); 2];
    tree.clip_interval(Interval::from((900, 1050)), &mut clippings);
}
