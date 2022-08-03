use crate::{collections::binary_tree::BinaryTree, GlowGL};


#[derive(Copy, Clone, PartialEq, Eq,Hash)]
struct GuiComponentPtr {
    data:u32,
}


pub struct FrameComponent{
    
}

pub struct GuiState {
    gl: GlowGL,
    gui_scene_graph: BinaryTree<GuiComponentPtr>,

}
