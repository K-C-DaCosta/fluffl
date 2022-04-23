use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, Index},
};

#[derive(Copy, Clone, Debug)]
pub struct GlobalInterval<V> {
    interval: Interval,
    data: V,
}
impl<V> Deref for GlobalInterval<V> {
    type Target = Interval;
    fn deref(&self) -> &Self::Target {
        &self.interval
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TreeInterval {
    clipped_interval: Interval,
    pub world_space_idx: usize,
}
impl Deref for TreeInterval {
    type Target = Interval;
    fn deref(&self) -> &Self::Target {
        &self.clipped_interval
    }
}

#[derive(Copy, Clone, Default)]
pub struct SearchIterFrame {
    root: usize,
    child_cursor: usize,
    visited: bool,
}

pub struct ScalarSearchIter<'a, V> {
    tree: &'a CircularSegmentTree<V>,
    node: usize,
    depth: u32,
    t: u128,
}
impl<'a, V> ScalarSearchIter<'a, V> {
    pub fn new(tree: &'a CircularSegmentTree<V>, t: u128) -> Self {
        Self {
            tree,
            node: 1,
            depth: 0,
            t,
        }
    }
}

impl<'a, V> Iterator for ScalarSearchIter<'a, V> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let depth = self.depth;
        let tree = &self.tree;
        let max_depth = tree.max_depth;
        let width = tree.width;
        let remainder_mask = width - 1;
        let node = self.node;
        let t = self.t;

        (depth < max_depth && tree.node_buckets.contains_key(&node)).then(|| {
            if compute_interval((node << 1) + 0, width).is_within(t & remainder_mask) {
                self.node = (node << 1) + 0;
            } else {
                self.node = (node << 1) + 1
            }
            self.depth += 1;
            node
        })
    }
}

pub struct FixedStack<V> {
    memory: [V; 64],
    cursor: usize,
}
impl<V> FixedStack<V>
where
    V: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            memory: [V::default(); 64],
            cursor: 0,
        }
    }

    pub fn push(&mut self, v: V) {
        self.memory[self.cursor.min(63)] = v;
        self.cursor = (self.cursor + 1).min(63);
    }

    pub fn pop(&mut self) -> Option<V> {
        (self.cursor > 0).then(|| {
            let popped_val = self.memory[self.cursor - 1];
            self.cursor -= 1;
            popped_val
        })
    }
}

pub struct IntervalSearchIter<'a, V> {
    tree: &'a CircularSegmentTree<V>,
    stack: FixedStack<SearchIterFrame>,
    circular_interval: Interval,
}
impl<'a, V> IntervalSearchIter<'a, V> {
    pub fn new(tree: &'a CircularSegmentTree<V>, circular_interval: Interval) -> Self {
        let mut stack = FixedStack::new();
        stack.push(SearchIterFrame {
            root: 1,
            child_cursor: 0,
            visited: false,
        });
        Self {
            tree,
            stack,
            circular_interval,
        }
    }
}
impl<'a, V> Iterator for IntervalSearchIter<'a, V> {
    type Item = Option<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|mut sf| {
            let root = sf.root;
            let cursor = sf.child_cursor;
            let child = (root << 1) + sf.child_cursor;

            let root_has_no_entries = self
                .tree
                .node_buckets
                .get(&root)
                .map(|a| a.is_empty())
                .unwrap_or_default();

            let yield_res = (sf.visited == false && root_has_no_entries == false).then(|| root);
            sf.visited = true;
            sf.child_cursor += 1;

            if cursor < 2 {
                self.stack.push(sf);
                let interval = compute_interval(child, self.tree.width);

                if self.tree.node_buckets.contains_key(&child)
                    && interval.is_overlapping(&self.circular_interval)
                {
                    self.stack.push(SearchIterFrame {
                        root: child,
                        child_cursor: 0,
                        visited: false,
                    });
                }
            }

            yield_res
        })
    }
}

/// A Special segment tree where root is segment: [0,width]
/// where width = 2^k for some k
/// all segments are interpreted to be circular
pub struct CircularSegmentTree<V> {
    max_depth: u32,
    //needs to be a power of two
    width: u128,
    exponent: u128,
    node_buckets: HashMap<usize, Vec<TreeInterval>>,
    global_inervals: Vec<GlobalInterval<V>>,
}

impl<V> CircularSegmentTree<V> {
    pub fn new(max_depth: u32, width: u128) -> Self {
        assert_eq!(width.count_ones(), 1, "ubound needs to be a power of two");
        Self {
            max_depth,
            width,
            exponent: (width - 1).count_ones() as u128,
            node_buckets: HashMap::new(),
            global_inervals: Vec::new(),
        }
    }

    pub fn search_scalar<'a>(&'a self, t: u128) -> impl Iterator<Item = &'a GlobalInterval<V>> {
        let tree = self;
        let time = t;
        let remainder_mask = self.width - 1;
        self.bucket_search_scalar(t)
            .flat_map(move |ptr| tree.node_buckets.get(&ptr))
            .flat_map(|list| list.iter())
            .filter(move |interval| interval.is_within(time & remainder_mask))
            .map(move |interval| unsafe {
                tree.global_inervals.get_unchecked(interval.world_space_idx)
            })
            .filter(move |gi| gi.is_within(time))
    }

    ///searches buckets that intersect `t`, where `t` is not assumed to be circular
    ///but gets converted to circular t internally 
    fn bucket_search_scalar<'a>(&'a self, t: u128) -> ScalarSearchIter<'a, V> {
        ScalarSearchIter::new(self, t)
    }

    ///returns all buckets that overlap `circular_interval`\
    ///`circular_interval` is assumed to be circular and is NOT converted to circular internally 
    fn bucket_interval_search<'a>(
        &'a self,
        circular_interval: Interval,
    ) -> IntervalSearchIter<'a, V> {
        IntervalSearchIter::new(&self, circular_interval)
    }

    pub fn insert(&mut self, interval: Interval, data: V) {
        let exponent = self.exponent;
        let remainder_mask = (1 << exponent) - 1;

        let Interval {
            start_time: lo,
            stop_time: hi,
        } = interval;

        let lo_block = lo >> exponent;
        let hi_block = hi >> exponent;

        self.global_inervals.push(GlobalInterval { interval, data });
        let alias = self.global_inervals.len() - 1;
        let num_blocks_interval_spans = (hi_block - lo_block) + 1;

        //clip the intervals and make them circular
        if num_blocks_interval_spans >= 3 {
            self.insert_helper(Interval::from((0, self.width)), alias, 1, 0, 0, self.width);
        } else if num_blocks_interval_spans >= 2 {
            self.insert_helper(
                Interval::from((lo & remainder_mask, self.width)),
                alias,
                1,
                0,
                0,
                self.width,
            );
            self.insert_helper(
                Interval::from((0, self.width - (hi & remainder_mask))),
                alias,
                1,
                0,
                0,
                self.width,
            );
        } else {
            self.insert_helper(
                Interval::from((lo & remainder_mask, hi & remainder_mask)),
                alias,
                1,
                0,
                0,
                self.width,
            );
        }
    }
    fn insert_helper(
        &mut self,
        interval: Interval,
        alias: usize,
        mut root: usize,
        mut depth: u32,
        mut lo: u128,
        mut hi: u128,
    ) {
        let tree_interval = TreeInterval {
            clipped_interval: interval,
            world_space_idx: alias,
        };

        while depth < self.max_depth {
            let mid = lo + (hi - lo) / 2;
            let overlapping_left = interval.is_overlapping(&Interval::from((lo, mid)));
            let overlapping_right = interval.is_overlapping(&Interval::from((mid, hi)));
            if overlapping_left != overlapping_right {
                if let None = self.node_buckets.get(&root) {
                    self.node_buckets.insert(root, vec![]);
                }

                //select root
                if overlapping_left {
                    root = root * 2 + 0;
                    hi = mid;
                } else {
                    root = root * 2 + 1;
                    lo = mid;
                }

                //increment depth counter
                depth += 1;
            } else {
                //cant go any further without splitting
                //so stop and insert below
                break;
            }
        }

        //insert
        match self.node_buckets.get_mut(&root) {
            Some(v) => v.push(tree_interval),
            None => {
                self.node_buckets.insert(root, vec![tree_interval]);
            }
        };
    }
}

/// Represents an interval
#[derive(Copy, Clone)]
pub struct Interval {
    pub start_time: u128,
    pub stop_time: u128,
}

impl Debug for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.start_time, self.stop_time)
    }
}

impl Interval {
    pub fn is_seperating(&self, other_interval: &Self) -> bool {
        let &Self {
            start_time: lo_a,
            stop_time: hi_a,
        } = self;

        let &Self {
            start_time: lo_b,
            stop_time: hi_b,
        } = other_interval;

        lo_b > hi_a || lo_a > hi_b
    }

    pub fn is_overlapping(&self, other_interval: &Self) -> bool {
        !self.is_seperating(other_interval)
    }
    pub fn is_within(&self, t: u128) -> bool {
        let &Self {
            start_time: lo,
            stop_time: hi,
        } = self;

        t >= lo && t <= hi
    }
}
impl From<(u128, u128)> for Interval {
    fn from((start_time, stop_time): (u128, u128)) -> Self {
        Self {
            start_time,
            stop_time,
        }
    }
}

pub trait HasAudioStream {
    fn frequency(&self) -> u64;
    fn channels(&self) -> u64;
    fn interval(&self) -> Interval;
    fn attack_time(&self) -> u128;
    fn release_time(&self) -> u128;
    fn is_dead(&self) -> bool;
    fn output_buffer(&self) -> &RingBuffer<512, f32>;
    fn output_buffer_mut(&mut self) -> &mut RingBuffer<512, f32>;
}

/// mixes sounds together
pub struct Mixer {
    sound_track: Vec<Interval>,
}
impl Mixer {
    pub fn with_track(mut sound_track: Vec<Interval>) -> Self {
        sound_track.sort_by_key(|&i| i.start_time);
        Self { sound_track }
    }

    /// fetches any track that fits within the time coordinate
    /// ### Notes:
    /// - complexity: `O(log(n))`
    pub fn get_any_track(&self, time: u128) -> Option<usize> {
        let mut lo = 0;
        let mut hi = self.sound_track.len() - 1;
        let mut left_most_interval = None;

        //binary search for the first interval that fits within the `time:u128` query
        //this is used as an initial starting point for the left-most search
        while lo <= hi {
            let mid = (hi - lo) / 2 + lo;
            let item = self.sound_track[mid];
            if item.is_within(time) {
                left_most_interval = Some(mid);
                break;
            } else if time < item.start_time {
                //take left subarray 
                hi = mid - 1;
            } else {
                //take right subarray
                lo = mid + 1;
            }
        }

        left_most_interval
    }

    /// fetches earliest track that intersects it
    /// ### Notes:
    /// - complexity: `O(log(n))`
    pub fn get_earliest_track(&self, time: u128) -> Option<usize> {
        let sound_track = &self.sound_track;
        let left_most_interval = self.get_any_track(time);

        // after the binary search you aren't nececiarily going to get the leftmost track that fits within `time`
        // so you have to do bisection-like iterations to get there quick
        if let Some(hi) = left_most_interval {
            let mut hi = hi;
            let mut lo = 0;
            loop {
                let mid = (hi - lo) / 2 + lo;
                let hi_in = sound_track[hi].is_within(time);
                let lo_in = sound_track[lo].is_within(time);
                let mid_in = sound_track[mid].is_within(time);
                if hi - lo <= 1 {
                    if lo_in {
                        return Some(lo);
                    }
                    if hi_in {
                        return Some(hi);
                    }
                } else if hi_in == lo_in {
                    return Some(lo);
                } else if mid_in != hi_in {
                    lo = mid;
                } else if lo_in != mid_in {
                    hi = mid;
                }
            }
        }

        left_most_interval
    }

    
}
impl Index<usize> for Mixer {
    type Output = Interval;
    fn index(&self, index: usize) -> &Self::Output {
        &self.sound_track[index]
    }
}

#[test]
fn interval_tree_test() {
    let mut tree = CircularSegmentTree::<()>::new(30, 1 << 32);
    tree.insert(Interval::from((0, 20)), ());
    tree.insert(Interval::from((32, 50)), ());
    tree.insert(Interval::from((2_000_000, 3_000_000)), ());
    tree.insert(Interval::from((4194304 + 1, 4194304 + 50_000)), ());

    let time = 4194304 + 1;

    for i in tree.search_scalar(time) {
        println!("{:?}", i);
    }
}

fn compute_interval(child: usize, width: u128) -> Interval {
    let child_level = log2_u64(child as u64) as u128;
    let level_block = child as u128 - (1 << child_level);
    let interval_length = width >> child_level;
    let start_time = interval_length * level_block;
    Interval {
        start_time,
        stop_time: start_time + interval_length,
    }
}

fn log2_u64(mut x: u64) -> u64 {
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x |= x >> 32;
    x.count_ones() as u64 - 1
}

#[allow(dead_code)]
fn log2_u128(mut x: u128) -> u128 {
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x |= x >> 32;
    x |= x >> 64;
    x.count_ones() as u128 - 1
}



// eventually i will use this to feed mixed audio to the audio backend  
pub struct RingBuffer<const N: usize, T> {
    memory: [T; N],
    front: u32,
    rear: u32,
    len: u32,
}
