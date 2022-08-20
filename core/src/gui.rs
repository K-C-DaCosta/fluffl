use std::{
    collections::{HashMap, VecDeque},
    fmt, vec,
};

use glow::HasContext;

use crate::{
    collections::{
        fixed_stack::FixedStack,
        flat_nary_tree::{LinearTree, NodeID, StackSignal},
        linked_list::{LinkedList, PackedLinkedList},
    },
    extras::ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder, OglProg},
    math::{self, stack::MatStack, translate4, ComponentWriter, Mat4, Vec2, Vec4, AABB2, FP32},
    mem::force_borrow_mut,
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

pub struct GUIManager {
    gl: GlowGL,
    position: Vec4<f32>,
    renderer: GuiRenderer,

    stack: MatStack<f32>,

    component_key_state: u32,

    ///component that is currently in "focus"
    focused_component: Option<GuiComponentKey>,

    ///component that the mouse is currently overlapping,but my not necessarily be in focus
    hover_component: Option<GuiComponentKey>,

    ///encodes the parent child relationship between nodes
    gui_component_tree: LinearTree<Box<dyn GUIComponent>>,
    key_to_aabb_table: HashMap<GuiComponentKey, AABB2<f32>>,
    component_signal_queue: VecDeque<components::ComponentEventSignal>,

    window_events: VecDeque<EventKind>,
}

impl GUIManager {
    pub fn new(gl: GlowGL) -> Self {
        let mut manager = Self {
            renderer: GuiRenderer::new(&gl),
            focused_component: None,
            hover_component: None,
            gui_component_tree: LinearTree::new(),
            component_key_state: 0,
            component_signal_queue: VecDeque::new(),
            window_events: VecDeque::new(),
            stack: MatStack::new(),
            position: Vec4::zero(),
            key_to_aabb_table: HashMap::new(),
            gl,
        };

        let root = manager.add_component(Box::new(Origin::new()), GuiComponentKey::default());

        let frame2 = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([200., 100.])
                    .with_roundness([0., 0., 10.0, 10.0])
                    .with_position([64.0, 400.0]),
            ),
            root,
        );

        let frame = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([400., 200.])
                    .with_roundness([0., 0., 30.0, 30.0])
                    .with_position([64.0, 32.0]),
            ),
            root,
        );

        let red_frame = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([128., 64.])
                    .with_color([0.7, 0.2, 0., 1.0])
                    .with_position([63.0, 33.0]),
            ),
            frame,
        );

        manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([32., 32.])
                    .with_color(Vec4::rgb_u32(0x277BC0))
                    .with_position([8.0, 8.0]),
            ),
            red_frame,
        );

        let orange_frame = manager.add_component(
            Box::new(
                Frame::new()
                    .with_bounds([256., 128.])
                    .with_color(Vec4::rgb_u32(0xFF7F3F))
                    .with_roundness(Vec4::from([1., 1., 30., 30.]))
                    .with_edge_color([0., 0., 0., 1.0])
                    .with_position([128.0, 64.0]),
            ),
            frame,
        );

        for k in 0..6 {
            let mut comp = Box::new(
                Frame::new()
                    .with_bounds([32., 32.])
                    .with_color(Vec4::rgb_u32(0x277BC0))
                    .with_roundness(Vec4::from([1., 1., 1., 1.]))
                    .with_edge_color([0., 0., 0., 1.0])
                    .with_position([10.0 + 35.0 * (k as f32), 10.0]),
            );
            manager.add_component(comp, orange_frame);
        }

        manager
    }

    pub fn add_component(
        &mut self,
        comp: Box<dyn GUIComponent>,
        parent: GuiComponentKey,
    ) -> GuiComponentKey {
        let id = self.gui_component_tree.add(comp, parent.into());
        let key = GuiComponentKey::from(id);
        key
    }

    pub fn push_event(&mut self, event: EventKind) {
        self.window_events.push_back(event);
    }

    fn handle_incoming_events(&mut self) {
        self.recompute_aabb_table();

        let window_events = &mut self.window_events;
        let key_to_aabb_table = &mut self.key_to_aabb_table;
        let gui_component_tree = &mut self.gui_component_tree;
        let focused_component = &mut self.focused_component;
        let hover_component = &mut self.hover_component;
        let component_signal_queue = &mut self.component_signal_queue;

        while let Some(event) = window_events.pop_front() {
            let old_signal_len = component_signal_queue.len();
            match event {
                EventKind::Resize { width, height } => {}
                EventKind::MouseUp { x, y, .. } => {
                    if let &mut Some(gui_comp_key) = focused_component {
                        component_signal_queue
                            .push_back(ComponentEventSignal::OnRelease(gui_comp_key, event));
                    }
                    *focused_component = None;
                }

                EventKind::MouseDown { x, y, .. } => {
                    let mouse_pos = Vec2::from([x as f32, y as f32]);
                    *focused_component = None;
                    for (key, aabb) in Self::aabb_iter(gui_component_tree, key_to_aabb_table) {
                        if aabb.is_point_inside(mouse_pos) {
                            *focused_component = Some(key);
                        }
                    }
                }

                EventKind::MouseMove { x, y, dx, dy } => {
                    let mouse_pos = Vec2::from([x as f32, y as f32]);
                    let _disp = Vec2::from([dx as f32, dy as f32]);

                    if let &mut Some(fkey) = focused_component {
                        component_signal_queue.push_back(ComponentEventSignal::Drag(fkey, event));

                        gui_component_tree.get_mut(fkey).unwrap().translate(_disp);
                    }

                    match hover_component {
                        // if something is being hovered check if mouse has left the component
                        &mut Some(current_hover_key) => {
                            let mut local_hover = None;

                            for (key, aabb) in
                                Self::aabb_iter(gui_component_tree, key_to_aabb_table)
                            {
                                if aabb.is_point_inside(mouse_pos) {
                                    local_hover = Some(key);
                                }
                            }

                            match local_hover {
                                Some(local_hover_key) => {
                                    //mouse has left the current component and has entered another component
                                    if local_hover_key != current_hover_key {
                                        component_signal_queue.push_back(
                                            ComponentEventSignal::HoverOut(
                                                current_hover_key,
                                                event,
                                            ),
                                        );

                                        component_signal_queue.push_back(
                                            ComponentEventSignal::HoverIn(local_hover_key, event),
                                        );

                                        *hover_component = Some(local_hover_key);
                                    }
                                }

                                //mouse has left the current component and is hovering over nothing
                                None => {
                                    component_signal_queue.push_back(
                                        ComponentEventSignal::HoverOut(current_hover_key, event),
                                    );

                                    //nothing is being hovered so set pointer to None
                                    *hover_component = None;
                                }
                            }
                        }

                        //if nothing is being hovered check if mouse is inside hovering a component
                        None => {
                            //run through aabbs in pre-order traversal
                            for (key, aabb) in
                                Self::aabb_iter(gui_component_tree, key_to_aabb_table)
                            {
                                if aabb.is_point_inside(mouse_pos) {
                                    *hover_component = Some(key);
                                }
                            }

                            if let &mut Some(key) = hover_component {
                                component_signal_queue
                                    .push_back(ComponentEventSignal::HoverIn(key, event));
                            }
                        }
                    }
                }
                _ => (),
            }

            // if component_signal_queue.len() > old_signal_len {
            //     println!("signal added: {:?}", &component_signal_queue.make_contiguous()[old_signal_len..] );
            // }
        }

        // if let Some(key) = local_focused {
        //     // self.key_to_component_table
        //     //     .get_mut(&key)
        //     //     .unwrap()
        //     //     .as_any_mut()
        //     //     .downcast_mut::<Frame>()
        //     //     .unwrap()
        //     //     .color = Vec4::rgb_u32(0xff0000);
        // }
    }
    fn send_component_signals() {}

    pub fn aabb_iter<'a>(
        gui_component_tree: &'a LinearTree<Box<dyn GUIComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
    ) -> impl Iterator<Item = (GuiComponentKey, AABB2<f32>)> + 'a {
        gui_component_tree.iter().map(move |node_info| {
            let &key = &GuiComponentKey::from(node_info.id);
            let &aabb = key_to_aabb_table.get(&key).unwrap();
            (key, aabb)
        })
    }

    pub fn recompute_aabb_table(&mut self) {
        self.key_to_aabb_table.clear();

        //force a split borrow, safe because key_to_aabb_table is never mutated in the component_global_position_top_down(..) function
        let key_to_aabb_table = unsafe { force_borrow_mut(&mut self.key_to_aabb_table) };

        for (_, key, mat) in self.component_global_position_top_down() {
            let aabb = {
                let comp = self.gui_component_tree.get(key).unwrap();
                let global_pos = mat * Vec4::from([0., 0., 0., 1.]);
                comp.get_aabb(global_pos)
            };
            key_to_aabb_table.insert(key, aabb);
        }
    }

    pub fn render(&mut self, window_width: f32, window_height: f32) {
        self.handle_incoming_events();

        let gl = &self.gl;
        let stack = &mut self.stack;

        let renderer = &self.renderer;
        let gui_component_tree = &self.gui_component_tree;
        let key_to_aabb_table = &self.key_to_aabb_table;

        let compute_global_position = |rel_pos, stack: &MatStack<f32>| {
            let s = stack;
            let pos = Vec4::to_pos(rel_pos);
            let &transform = s.peek();
            transform * pos
        };

        let build_state = |global_position| RenderState {
            global_position,
            renderer,
            gui_component_tree,
            key_to_aabb_table,
        };

        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        stack.clear();
        for (sig, _key, comp) in gui_component_tree.iter_stack_signals() {
            let &rel_pos = comp.rel_position();
            let transform = translate4(Vec4::to_pos(rel_pos));
            // println!("sig:{:?}",sig);
            match sig {
                StackSignal::Nop => {
                    stack.pop();
                    let gpos = compute_global_position(rel_pos, stack);
                    comp.render(gl, build_state(gpos), window_width, window_height);
                    stack.push(transform);
                }
                StackSignal::Pop { n_times } => {
                    stack.pop_multi(n_times + 1);
                    let gpos = compute_global_position(rel_pos, stack);
                    comp.render(gl, build_state(gpos), window_width, window_height);
                    stack.push(transform);
                }
                StackSignal::Push => {
                    let gpos = compute_global_position(rel_pos, stack);
                    comp.render(gl, build_state(gpos), window_width, window_height);
                    stack.push(transform);
                }
            }
        }

        unsafe {
            gl.disable(glow::BLEND);
        }
    }

    fn component_global_position_top_down(
        &self,
    ) -> impl Iterator<Item = (StackSignal, GuiComponentKey, Mat4<f32>)> + '_ {
        let mut s = MatStack::new();
        self.gui_component_tree
            .iter_stack_signals()
            .map(move |(sig, key, comp)| {
                let &comp_pos = comp.rel_position();
                let transform = translate4(Vec4::to_pos(comp_pos));
                match sig {
                    StackSignal::Nop => {
                        s.pop();
                        s.push(transform);
                    }
                    StackSignal::Pop { n_times } => {
                        s.pop_multi(n_times + 1);
                        s.push(transform);
                    }
                    StackSignal::Push => {
                        s.push(transform);
                    }
                }
                (sig, GuiComponentKey::from(key), *s.peek())
            })
    }

    fn component_global_position_bottom_up<CB>(&self, mut cb: CB)
    where
        CB: FnMut(GuiComponentKey, Mat4<f32>) -> bool,
    {
        let mut mat_stack = MatStack::new();
        let mut key_stack = FixedStack::<32, GuiComponentKey>::new();

        let mut it = self.gui_component_tree.iter_stack_signals();

        let mut peek_and_prop = |matstack: &MatStack<_>, keystack: &FixedStack<32, _>| {
            let &mat = matstack.peek();
            let key = keystack.peek();
            cb(key, mat)
        };

        while let Some((sig, key, comp)) = it.next() {
            let &comp_pos = comp.rel_position();
            let transform = translate4(Vec4::to_pos(comp_pos));

            match sig {
                StackSignal::Nop => {
                    if peek_and_prop(&mat_stack, &key_stack) {
                        return;
                    }
                    mat_stack.pop();
                    key_stack.pop();

                    mat_stack.push(transform);
                    key_stack.push(key.into());
                }
                StackSignal::Pop { n_times } => {
                    for _ in 0..n_times + 1 {
                        if peek_and_prop(&mat_stack, &key_stack) {
                            return;
                        }
                        mat_stack.pop();
                        key_stack.pop();
                    }

                    mat_stack.push(transform);
                    key_stack.push(key.into());
                }
                StackSignal::Push => {
                    mat_stack.push(transform);
                    key_stack.push(key.into());
                }
            }
        }

        //flush remaining items in stack
        while key_stack.len() > 0 {
            if peek_and_prop(&mat_stack, &key_stack) {
                return;
            }
            mat_stack.pop();
            key_stack.pop();
        }
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
