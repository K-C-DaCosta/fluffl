use std::hash::Hash;

use super::*;

#[derive(Copy, Clone, Default)]
pub struct TouchStats {
    pub prev_pos: [f32; 2],
    pub displacement: [f32; 2],
}
impl TouchStats {
    pub fn initial_stats(pos: [f32; 2]) -> Self {
        Self {
            prev_pos: pos,
            displacement: [0.0; 2],
        }
    }
}

#[derive(Default)]
pub struct TouchTracker<ID> {
    table: HashMap<ID, TouchStats>,
}

impl<ID> std::ops::Deref for TouchTracker<ID> {
    type Target = HashMap<ID, TouchStats>;
    fn deref(&self) -> &Self::Target {
        &self.table
    }
}
impl<ID> std::ops::DerefMut for TouchTracker<ID> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}

impl<ID> TouchTracker<ID>
where
    ID: Copy + Clone + PartialEq + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// # Description
    /// Tracks a position displacement
    /// # returns
    /// - touch displacement of `id`
    pub fn get_touch_displacement(&mut self, id: ID, new_pos: [f32; 2]) -> [f32; 2] {
        let touch_table = self;

        let old_pos = touch_table
            .get(&id)
            .map(|&x| x.prev_pos)
            .unwrap_or([0., 0.]);
        let disp = [new_pos[0] - old_pos[0], new_pos[1] - old_pos[1]];

        touch_table
            .table
            .entry(id)
            .and_modify(|stats| {
                stats.prev_pos = new_pos;
                stats.displacement = disp;
            })
            .or_insert_with(|| TouchStats::initial_stats(new_pos));

        touch_table.get(&id).unwrap().displacement
    }
}
