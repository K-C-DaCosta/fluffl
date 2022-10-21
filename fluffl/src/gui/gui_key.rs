use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct GuiComponentKey(pub usize);

impl GuiComponentKey {
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<NodeID> for GuiComponentKey {
    fn from(nid: NodeID) -> Self {
        unsafe { std::mem::transmute_copy(&nid) }
    }
}

impl From<GuiComponentKey> for NodeID {
    fn from(key: GuiComponentKey) -> Self {
        unsafe { std::mem::transmute_copy(&key) }
    }
}

impl fmt::Display for GuiComponentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T> std::ops::Index<GuiComponentKey> for Vec<T> {
    type Output = T;
    fn index(&self, idx: GuiComponentKey) -> &Self::Output {
        &self[idx.as_usize()]
    }
}

impl<T> std::ops::IndexMut<GuiComponentKey> for Vec<T> {
    fn index_mut(&mut self, idx: GuiComponentKey) -> &mut Self::Output {
        &mut self[idx.as_usize()]
    }
}
