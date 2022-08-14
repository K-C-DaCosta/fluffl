use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

use crate::{
    collections::{
        flat_nary_tree::{LinearTree, NodeID},
        linked_list::{LinkedList, PackedLinkedList},
    },
    math::{Vec2, Vec4, FP32},
    window::event_util::EventKind,
    GlowGL,
};


pub mod components; 


#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GuiComponentKey(u32);

impl fmt::Display for GuiComponentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

pub struct GUIManager {
    gl: GlowGL,

    component_key_state: u32,

    focused_component: Option<GuiComponentKey>,

    ///encodes the parent child relationship between nodes
    gui_component_tree: LinearTree<GuiComponentKey>,

    ///stores the position given component key
    key_to_node_table: HashMap<GuiComponentKey, NodeID>,

    component_signal_queue: VecDeque<components::ComponentEventSignal>,
}

impl GUIManager {
    pub fn new(gl: GlowGL) -> Self {
        Self {
            gl,
            focused_component: None,
            gui_component_tree: LinearTree::new(),
            key_to_node_table: HashMap::new(),
            component_key_state: 0,
            component_signal_queue: VecDeque::new(),
        }
    }
    pub fn render(window_Width: f32, window_height: f32) {

    }
    fn gen_component_key(&mut self) -> GuiComponentKey {
        let generated_key = GuiComponentKey(self.component_key_state);

        //increment state
        self.component_key_state += 1;

        generated_key
    }


}
