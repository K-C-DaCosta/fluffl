use std::{
    collections::{HashMap, VecDeque},
    fmt, vec,
};

use glow::HasContext;

use crate::{
    collections::{
        flat_nary_tree::{LinearTree, NodeID, StackSignal},
        linked_list::{LinkedList, PackedLinkedList},
    },
    extras::ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder, OglProg},
    math::{self, stack::MatStack, translate4, ComponentWriter, Mat4, Vec2, Vec4, FP32},
    window::event_util::EventKind,
    GlowGL,
};

pub mod components;
pub mod renderer;

use self::{components::*, renderer::*};

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum GuiShaderKind {
    Frame = 0,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct GuiComponentKey(u32);

impl fmt::Display for GuiComponentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

pub struct GUIManager {
    gl: GlowGL,
    position: Vec4<f32>,
    renderer: GuiRenderer,

    stack: MatStack<f32>,

    component_key_state: u32,

    focused_component: Option<GuiComponentKey>,

    ///encodes the parent child relationship between nodes
    gui_component_tree: LinearTree<GuiComponentKey>,

    ///stores the position given component key
    key_to_node_table: HashMap<GuiComponentKey, NodeID>,

    ///used to resolve component key to `GUIComponent`
    key_to_component_table: HashMap<GuiComponentKey, Box<dyn GUIComponent>>,

    component_signal_queue: VecDeque<components::ComponentEventSignal>,

    window_events: VecDeque<EventKind>,
}

impl GUIManager {
    pub fn new(gl: GlowGL) -> Self {
        let mut manager = Self {
            renderer: GuiRenderer::new(&gl),
            focused_component: None,
            gui_component_tree: LinearTree::new(),
            key_to_node_table: HashMap::new(),
            component_key_state: 0,
            component_signal_queue: VecDeque::new(),
            window_events: VecDeque::new(),
            stack: MatStack::new(),
            position: Vec4::zero(),
            key_to_component_table: HashMap::new(),
            gl,
        };

        let root = manager.add_component(Box::new(Origin::new()), NodeID::default());

        let frame = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([400., 200.])
                    .with_roundness([0., 0., 30.0, 30.0])
                    .with_position([64.0, 32.0]),
            ),
            root.1,
        );

        let red_frame = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([128., 64.])
                    .with_color([0.7, 0.2, 0., 1.0])
                    .with_position([63.0, 33.0]),
            ),
            frame.1,
        );

        manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([32., 32.])
                    .with_color(Vec4::rgb_u32(0x277BC0))
                    .with_position([8.0, 8.0]),
            ),
            red_frame.1,
        );

        let orange_frame = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([256., 128.])
                    .with_color(Vec4::rgb_u32(0xFF7F3F))
                    .with_roundness(Vec4::from([1., 1., 30., 30.]))
                    .with_edge_color([0.,0.,0.,1.0])
                    .with_position([128.0, 64.0]),
            ),
            frame.1,
        ); 

        for k in 0..6 {

            manager.add_component(
                Box::new(
                    Frame::new()
                        .with_bounds([32., 32.])
                        .with_color(Vec4::rgb_u32(0x277BC0))
                        .with_roundness(Vec4::from([1., 1., 1., 1.]))
                        .with_edge_color([0.,0.,0.,1.0])
                        .with_position([10.0 + 35.0 * (k as f32)  , 10.0]),
                ),
                orange_frame.1,
            );
        }

        manager.focused_component = Some(frame.0);

        manager
    }

    pub fn add_component(
        &mut self,
        comp: Box<dyn GUIComponent>,
        parent: NodeID,
    ) -> (GuiComponentKey, NodeID) {
        let key = self.gen_component_key();
        let id = self.gui_component_tree.add(key, parent);
        self.key_to_component_table.insert(key, comp);
        self.key_to_node_table.insert(key, id);
        (key, id)
    }

    pub fn push_event(&mut self, event: EventKind) {
        self.window_events.push_back(event);
    }

    pub fn render(&mut self, window_width: f32, window_height: f32) {
        let gl = &self.gl;

        while let Some(event) = self.window_events.pop_front() {
            match event {
                EventKind::Resize { width, height } => {
                    // self.uniforms.update_proj(
                    //     &gl,
                    //     &self.gui_shader_program,
                    //     width as f32,
                    //     height as f32,
                    // );
                }
                EventKind::MouseMove { x, y, .. } => {
                    let pos = Vec2::from([x as f32, y as f32]);

                    if let Some(fkey) = self.focused_component {
                        self.key_to_component_table
                            .get_mut(&fkey)
                            .unwrap()
                            .set_rel_position(pos);
                    }
                }
                _ => (),
            }
        }

        let s = &mut self.stack;
        let r = &mut self.renderer;
        let gui_component_tree = &mut self.gui_component_tree;
        let key_to_component_table = &mut self.key_to_component_table;

        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }



        s.clear();
        for (sig, &mut key) in gui_component_tree.iter_mut_stack_signals() {
            let comp = key_to_component_table.get(&key).unwrap();
            let &comp_pos = comp.rel_position();
            let transform = translate4(Vec4::to_pos(comp_pos));
            // println!("sig:{:?}",sig);
            match sig {
                StackSignal::Nop => {
                    s.pop();
                    comp.render(gl, r, s, window_width, window_height);
                    s.push(transform);
                }
                StackSignal::Pop { n_times } => {
                    s.pop_multi(n_times+1);
                    comp.render(gl, r, s, window_width, window_height);
                    s.push(transform);
                }
                StackSignal::Push => {
                    comp.render(gl, r, s, window_width, window_height);
                    s.push(transform);
                }
            }
        }
        unsafe {
            gl.disable(glow::BLEND);
        }
    }

    fn component_global_position_iter(&self) -> impl Iterator<Item=(StackSignal,GuiComponentKey,Mat4<f32>)> + '_ {
        let mut s = MatStack::new();
        let key_to_component_table = &self.key_to_component_table;        
        self.gui_component_tree.iter_stack_signals().map(move |(sig,&key)| {             
            let comp = key_to_component_table.get(&key).unwrap();
            let &comp_pos = comp.rel_position();
            let transform = translate4(Vec4::to_pos(comp_pos));
            match sig {
                StackSignal::Nop => {
                    s.pop();
                    s.push(transform);
                }
                StackSignal::Pop { n_times } => {
                    s.pop_multi(n_times+1);
                    s.push(transform);
                }
                StackSignal::Push => {
                    s.push(transform);
                }
            }
            (sig,key,*s.peek())
        })
    }

    fn gen_component_key(&mut self) -> GuiComponentKey {
        let generated_key = GuiComponentKey(self.component_key_state);

        //increment state
        self.component_key_state += 1;

        generated_key
    }
}

fn write_rectangle(component_list: &mut Vec<f32>, x0: Vec4<f32>, w: f32, h: f32) {
    let mut writer = ComponentWriter::from(component_list);

    let dx = Vec4::from_array([w, 0.0, 0.0, 0.0]);
    let dy = Vec4::from_array([0.0, h, 0.0, 0.0]);
    let tl = x0;
    let tr = x0 + dx;
    let bl = x0 + dy;
    let br = x0 + dx + dy;

    writer.write(&tl);
    writer.write(&tr);
    writer.write(&bl);

    writer.write(&tr);
    writer.write(&br);
    writer.write(&bl);
}
