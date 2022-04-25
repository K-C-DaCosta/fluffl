#![allow(dead_code)]

use crate::collections::{
    binary_tree::{BinaryTree, Ptr},
    fixed_stack::FixedStack,
};
use std::{
    fmt::Debug,
    ops::{Deref, Index, IndexMut},
};

mod index_types;
mod interval;
mod iterators;
mod pools;

use index_types::*;
use interval::*;
use iterators::*;
use pools::*;

pub use interval::{GlobalInterval, Interval};

/// A Special segment tree where root is segment: [0,width]
/// all segments are interpreted to be circular
pub struct SparseCircularSegmentTree<V> {
    max_depth: u32,
    //needs to be a power of two
    width: u128,
    exponent: u128,
    linear_tree: BinaryTree<BucketIndex>,
    global_pool: GlobalIntervalPool<V>,
    bucket_pool: BucketPool,
}

impl<V> SparseCircularSegmentTree<V> {
    pub fn new(max_depth: u32, width: u128) -> Self {
        assert_eq!(width.count_ones(), 1, "width needs to be a power of two");

        let mut bucket_pool = BucketPool::new();
        let mut linear_tree = BinaryTree::new();
        let root = linear_tree.allocate(Ptr::null(), bucket_pool.allocate());
        //make sure to root always exists
        linear_tree.set_root(root);

        Self {
            max_depth,
            width,
            exponent: (width - 1).count_ones() as u128,
            linear_tree,
            bucket_pool,
            global_pool: GlobalIntervalPool::new(),
        }
    }

    pub fn search_scalar<'a>(&'a self, t: u128) -> impl Iterator<Item = &'a GlobalInterval<V>> {
        let tree = self;
        let time = t;
        let remainder_mask = self.width - 1;
        self.bucket_search_scalar(t)
            // .map(move |ptr| {
            //     let i = compute_interval(ptr, self.width);
            //     println!("{} => {:?}", ptr, i);
            //     ptr
            // })
            .flat_map(move |ptr| tree.linear_tree[ptr].data)
            .flat_map(move |bucket_idx| tree.bucket_pool[bucket_idx].iter())
            .filter(move |interval| interval.is_within(time & remainder_mask))
            .filter_map(move |interval| tree.global_pool[interval.global_idx].as_ref())
            .filter(move |gi| gi.is_within(time))
    }

    ///searches buckets that intersect `t`, where `t` is not assumed to be circular
    ///but gets converted to circular t internally
    fn bucket_search_scalar<'a>(&'a self, t: u128) -> ScalarSearchIter<'a, V> {
        ScalarSearchIter::new(self, t)
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
            .into_iter()
            .flat_map(move |&i| IntervalSearchIter::new(&self, i).filter_map(|ptr| ptr))
    }

    pub fn insert(&mut self, interval: Interval, data: V) {
        let alias = self.global_pool.allocate(GlobalInterval { interval, data });

        //clip the intervals and make them circular
        let mut clipping_results = [Interval::default(); 2];
        let num_clips = self.clip_interval(interval, &mut clipping_results);

        for &interval in &clipping_results[0..num_clips] {
            self.insert_helper(interval, alias, self.linear_tree.root(), 0, 0, self.width);
        }
    }

    fn insert_helper(
        &mut self,
        interval: Interval,
        alias: GlobalIndex,
        mut root: Ptr,
        mut depth: u32,
        mut lo: u128,
        mut hi: u128,
    ) {
        let bucket_pool = &mut self.bucket_pool;
        let linear_tree = &mut self.linear_tree;

        let tree_interval = TreeInterval {
            clipped_interval: interval,
            global_idx: alias,
        };

        while depth < self.max_depth {
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
                depth += 1;
            } else {
                //cant go any further without splitting
                //so stop and insert below
                break;
            }
        }

        //insert
        let bucket_idx = linear_tree[root].data.unwrap();
        bucket_pool[bucket_idx].push(tree_interval);
    }

    pub fn remove(&mut self, query_interval: Interval) -> Option<GlobalInterval<V>> {
        let mut clip_results = [Interval::default(); 2];
        let num_clips = self.clip_interval(query_interval, &mut clip_results);

        let mut nodes_to_delete = FixedStack::<3, (Ptr, TreeInterval)>::new();

        for &clipped_interval in &clip_results[0..num_clips] {
            let midpoint = clipped_interval.midpoint();
            self.bucket_search_scalar(midpoint)
                .filter_map(|ptr| Some(ptr).zip(self.linear_tree[ptr].data))
                .map(|(ptr, bucket_index)| (ptr, &self.bucket_pool[bucket_index]))
                .flat_map(|(ptr, interval_list)| interval_list.iter().map(move |&i| (ptr, i)))
                .filter(|&(_ptr, tree_interval)| {
                    tree_interval.is_within(midpoint)
                        && self.global_pool[tree_interval.global_idx]
                            .as_ref()
                            .unwrap()
                            .interval
                            == query_interval
                })
                .for_each(|(ptr, tree_interval)| nodes_to_delete.push((ptr, tree_interval)))
        }

        if nodes_to_delete.len() > 0 {
            let (_, TreeInterval { global_idx, .. }) = nodes_to_delete[0];
            let value = self
                .global_pool
                .free(global_idx)
                .expect("value should be there");

            while let Some((ptr, tree_interval)) = nodes_to_delete.pop() {
                self.remove_helper(ptr, tree_interval)
            }

            return Some(value);
        }

        None
    }

    fn remove_helper(&mut self, mut root: Ptr, tree_interval: TreeInterval) {
        let bucket_idx = self.linear_tree[root].data.unwrap();

        //search for the tree interval index in the bucket
        let tree_interval_index = self.bucket_pool[bucket_idx]
            .iter()
            .enumerate()
            .find(|(_, e)| e.clipped_interval == tree_interval.clipped_interval)
            .map(|(i, _)| i)
            .expect("item should exist");

        //remove interval from bucket
        self.bucket_pool[bucket_idx].remove(tree_interval_index);

        while root != Ptr::null()
            && self.bucket_pool[bucket_idx].is_empty()
            && self.linear_tree[root].is_leaf()
        {
            let parent = self.linear_tree[root].parent;
            self.linear_tree.free(root);
            self.bucket_pool.free(bucket_idx);
            root = parent;
        }
    }

    /// returns number of clips
    fn clip_interval(&self, interval: Interval, clippings: &mut [Interval; 2]) -> usize {
        let exponent = self.exponent;
        let remainder_mask = (1 << exponent) - 1;

        let Interval { lo, hi } = interval;

        let lo_block = lo >> exponent;
        let hi_block = hi >> exponent;
        let num_blocks_interval_spans = (hi_block - lo_block) + 1;

        //clip the intervals and make them circular
        if num_blocks_interval_spans >= 3 {
            //this case the interval spans multiple blocks so insert it at the root and stop there
            clippings[0] = Interval::from((0, self.width));
            1
        } else if num_blocks_interval_spans >= 2 {
            //this interval spans two blocks so it can be broken up more efficiently
            clippings[0] = Interval::from((lo & remainder_mask, self.width));
            clippings[1] = Interval::from((0, hi & remainder_mask));
            2
        } else {
            //interval doesn't sit on a block boundary so clipping is not needed
            clippings[0] = Interval::from((lo & remainder_mask, hi & remainder_mask));
            1
        }
    }
}

#[test]
fn segment_tree_query_test_0() {
    let mut state = 0xaaabbu128;
    //generate intervals
    let mut intervals = (0..50_000)
        .map(|_| {
            let l = rand_lehmer64(&mut state) as u128 % 3_600_000;
            let u = 1 + l + rand_lehmer64(&mut state) as u128 % 60_000;
            (l, u)
        })
        .map(|a| Interval::from(a))
        .collect::<Vec<_>>();
    intervals.sort_by(|a, b| {
        if a.lo == b.lo {
            a.hi.cmp(&b.hi)
        } else {
            a.lo.cmp(&b.lo)
        }
    });
    // println!("{:?}",intervals);
    let lbound = intervals.iter().min_by_key(|a| a.lo).unwrap().lo;
    let ubound = intervals.iter().max_by_key(|a| a.hi).unwrap().hi;

    //create tree
    let mut tree = SparseCircularSegmentTree::<()>::new(30, 1 << 30);

    //insert intervals into tree
    for &range in &intervals {
        tree.insert(range, ());
    }

    let mut time = lbound;
    let step_size = ((ubound - lbound) / 5000).max(1);
    let mut tree_search_results: Vec<Interval> = Vec::with_capacity(500);
    let mut linear_search_results: Vec<Interval> = Vec::with_capacity(500);

    let mut num_times_tree_beats_linear = 0;
    let mut total_searches = 0;

    let mut tree_avg_dt = 0;
    let mut linear_avg_dt = 0;

    //start at time = lbound and step by fixed size to ubound
    while time <= ubound {
        linear_search_results.clear();
        tree_search_results.clear();

        //add search results for the tree
        let t0 = std::time::Instant::now();
        for i in tree.search_scalar(time) {
            tree_search_results.push(i.interval);
        }
        let tree_dt = t0.elapsed().as_millis();

        //add search results for linear search
        let t0 = std::time::Instant::now();
        for &i in intervals.iter().filter(|i| i.is_within(time)) {
            linear_search_results.push(i);
        }
        let linear_dt = t0.elapsed().as_millis();

        if tree_dt <= linear_dt {
            num_times_tree_beats_linear += 1;
        }
        total_searches += 1;

        tree_avg_dt += tree_dt;
        linear_avg_dt += linear_dt;

        linear_search_results.sort_by(|a, b| {
            if a.lo == b.lo {
                a.hi.cmp(&b.hi)
            } else {
                a.lo.cmp(&b.lo)
            }
        });
        tree_search_results.sort_by(|a, b| {
            if a.lo == b.lo {
                a.hi.cmp(&b.hi)
            } else {
                a.lo.cmp(&b.lo)
            }
        });

        //compare tree results agaisnt linear search
        //both arrays should be exactly the same
        assert_eq!(
            linear_search_results,
            tree_search_results,
            "t = {} (linear_len:{}| tree_len:{})",
            time,
            linear_search_results.len(),
            tree_search_results.len()
        );

        time += step_size;
    }

    println!(
        "tree wins {}  times out of a total of {} searches\n\n",
        num_times_tree_beats_linear, total_searches,
    );

    println!(
        "linear total elapsed :{} ms , tree total elapsed: {} ms ",
        linear_avg_dt, tree_avg_dt
    );

    println!(
        "linear mean :{} ms , tree mean: {} ms ",
        linear_avg_dt / total_searches,
        tree_avg_dt / total_searches
    );
}

pub fn rand_lehmer64(state: &mut u128) -> u64 {
    *state = state.wrapping_mul(0xda942042e4dd58b5);
    (*state >> 64) as u64
}
