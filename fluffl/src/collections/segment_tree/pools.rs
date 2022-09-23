#![allow(dead_code)]

use super::*;

pub struct GlobalIntervalPool<V> {
    pool: Vec<Option<GlobalInterval<V>>>,
    free_slots: Vec<GlobalIndex>,
}

impl<V> GlobalIntervalPool<V> {
    pub fn new() -> Self {
        Self {
            pool: Vec::new(),
            free_slots: Vec::new(),
        }
    }

    pub fn allocate(&mut self, int: GlobalInterval<V>) -> GlobalIndex {
        if let Some(idx) = self.free_slots.pop() {
            self[idx] = Some(int);
            idx
        } else {
            self.pool.push(Some(int));
            GlobalIndex::from(self.pool.len() - 1)
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a Option<GlobalInterval<V>>> {
        self.pool.iter()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<GlobalInterval<V>>> {
        self.pool.iter_mut()
    }

    pub fn free(&mut self, idx: GlobalIndex) -> Option<GlobalInterval<V>> {
        self.free_slots.push(idx);
        self[idx].take()
    }

    pub fn free_slots(&self) -> &[GlobalIndex] {
        self.free_slots.as_slice()
    }
    pub fn pool(&self) -> &[Option<GlobalInterval<V>>] {
        self.pool.as_slice()
    }
}
impl<V> Index<GlobalIndex> for GlobalIntervalPool<V> {
    type Output = Option<GlobalInterval<V>>;
    fn index(&self, index: GlobalIndex) -> &Self::Output {
        &self.pool[index.idx]
    }
}
impl<V> IndexMut<GlobalIndex> for GlobalIntervalPool<V> {
    fn index_mut(&mut self, index: GlobalIndex) -> &mut Self::Output {
        &mut self.pool[index.idx]
    }
}

pub struct BucketPool {
    /// a list of interval buckets
    pool_list: Vec<Vec<TreeInterval>>,
    /// a list of unused pools
    free_pools: Vec<BucketIndex>,
}

impl BucketPool {
    pub fn new() -> Self {
        Self {
            pool_list: Vec::new(),
            free_pools: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> BucketIndex {
        //pop from free pools stack if free nodes exists
        if let Some(idx) = self.free_pools.pop() {
            idx
        } else {
            self.pool_list.push(Vec::with_capacity(16));
            BucketIndex::from_usize(self.pool_list.len() - 1)
        }
    }

    pub fn pool(&self) -> &[Vec<TreeInterval>] {
        self.pool_list.as_slice()
    }

    pub fn free_pools(&self) -> &[BucketIndex] {
        self.free_pools.as_slice()
    }

    #[allow(dead_code)]
    pub fn allocate_with(&mut self, interval: TreeInterval) -> BucketIndex {
        let idx = self.allocate();
        self[idx].push(interval);
        idx
    }

    pub fn free(&mut self, idx: BucketIndex) {
        self[idx].clear();
        self.free_pools.push(idx);
    }
}
impl std::ops::Index<BucketIndex> for BucketPool {
    type Output = Vec<TreeInterval>;
    fn index(&self, index: BucketIndex) -> &Self::Output {
        unsafe { self.pool_list.get_unchecked(index.idx) }
    }
}
impl std::ops::IndexMut<BucketIndex> for BucketPool {
    fn index_mut(&mut self, index: BucketIndex) -> &mut Self::Output {
        unsafe { self.pool_list.get_unchecked_mut(index.idx) }
    }
}
