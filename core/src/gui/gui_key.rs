use super::*; 

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct GuiComponentKey(usize);

impl From<NodeID> for GuiComponentKey {
    fn from(nid: NodeID) -> Self {
        unsafe { std::mem::transmute_copy(&nid) }
    }
}
impl Into<NodeID> for GuiComponentKey {
    fn into(self) -> NodeID {
        unsafe { std::mem::transmute_copy(&self) }
    }
}

impl fmt::Display for GuiComponentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}