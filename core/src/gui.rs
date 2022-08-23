use std::{
    collections::{HashMap, VecDeque},
    fmt, vec,
};

use glow::HasContext;

use crate::{
    collections::{
        fixed_stack::FixedStack,
        flat_nary_tree::{LinearTree, NodeID, StackSignal},
    },
    extras::{
        ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder, OglProg},
        text_writer::TextWriter,
    },
    math::{self, stack::MatStack, translate4, ComponentWriter, Mat4, Vec2, Vec4, AABB2},
    mem::force_borrow_mut,
    window::event_util::EventKind,
    GlowGL,
};

mod builders;
mod components;
mod gui_key;
mod handler_block;
mod renderer;

use self::{builders::*, components::*, gui_key::*, handler_block::*, renderer::*};

pub type ListenerCallBack<ProgramState> =
    Box<dyn FnMut(EventListenerInfo<'_, ProgramState>) -> Option<()>>;

pub struct GuiManager<ProgramState> {
    gl: GlowGL,
    state: Option<ProgramState>,
    renderer: GuiRenderer,
    stack: MatStack<f32>,

    ///component that is currently in "focus"
    focused_component: Option<GuiComponentKey>,
    ///component that the mouse is currently overlapping,but my not necessarily be in focus
    hover_component: Option<GuiComponentKey>,

    ///encodes the parent child relationship between nodes
    gui_component_tree: LinearTree<Box<dyn GuiComponent>>,

    // key_to_event_handlers_table: HashMap<GuiComponentKey>
    key_to_aabb_table: HashMap<GuiComponentKey, AABB2<f32>>,

    key_to_handler_block_table: HashMap<GuiComponentKey, ComponentHandlerBlock<ProgramState>>,

    component_signal_queue: VecDeque<components::ComponentEventSignal>,

    window_events: VecDeque<EventKind>,
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn new(gl: GlowGL) -> Self {
        Self {
            renderer: GuiRenderer::new(&gl),
            focused_component: None,
            hover_component: None,
            gui_component_tree: LinearTree::new(),
            component_signal_queue: VecDeque::new(),
            window_events: VecDeque::new(),
            stack: MatStack::new(),
            key_to_aabb_table: HashMap::new(),
            key_to_handler_block_table: HashMap::new(),
            gl,
            state: None,
        }
        .setup_test_gui()
    }

    pub fn init_state(&mut self, state: ProgramState) {
        self.state = Some(state);
    }

    pub fn set_listener(
        &mut self,
        key: GuiComponentKey,
        listener: ComponentEventListener<ProgramState>,
    ) {
        let key_to_handler_block_table = &mut self.key_to_handler_block_table;
        key_to_handler_block_table
            .get_mut(&key)
            .expect("key missing")
            .set_handler(listener);
    }

    pub fn add_component_with_builder<'a, CompKind: GuiComponent + 'static>(
        &'a mut self,
    ) -> ComponentBuilder<'a, CompKind, ProgramState> {
        ComponentBuilder::new(self)
    }

    pub fn add_component(
        &mut self,
        parent: GuiComponentKey,
        comp: Box<dyn GuiComponent>,
    ) -> GuiComponentKey {
        let id = self.gui_component_tree.add(comp, parent.into());
        let key = GuiComponentKey::from(id);
        self.key_to_handler_block_table
            .insert(key, ComponentHandlerBlock::new());
        key
    }

    pub fn push_event(&mut self, event: EventKind) {
        self.window_events.push_back(event);
    }

    pub fn render(&mut self, text_writer: &mut TextWriter, window_width: f32, window_height: f32) {
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
                    comp.render(
                        gl,
                        build_state(gpos),
                        text_writer,
                        window_width,
                        window_height,
                    );
                    stack.push(transform);
                }
                StackSignal::Pop { n_times } => {
                    stack.pop_multi(n_times + 1);
                    let gpos = compute_global_position(rel_pos, stack);
                    comp.render(
                        gl,
                        build_state(gpos),
                        text_writer,
                        window_width,
                        window_height,
                    );
                    stack.push(transform);
                }
                StackSignal::Push => {
                    let gpos = compute_global_position(rel_pos, stack);
                    comp.render(
                        gl,
                        build_state(gpos),
                        text_writer,
                        window_width,
                        window_height,
                    );
                    stack.push(transform);
                }
            }
        }

        unsafe {
            gl.disable(glow::BLEND);
        }
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    fn setup_test_gui(self) -> Self {
        let mut manager = self;
        let root = manager.add_component(GuiComponentKey::default(), Box::new(Origin::new()));

        let alt_frame = manager
            .add_component_with_builder()
            .with_component(
                Frame::new()
                    .with_bounds([200., 100.])
                    .with_roundness([0., 0., 10.0, 10.0])
                    .with_position([64.0, 400.0]),
            )
            .with_parent(root)
            .with_drag(true)
            .build();

        let frame = manager
            .add_component_with_builder()
            .with_component(
                Frame::new()
                    .with_bounds([400., 200.])
                    .with_roundness([0., 0., 30.0, 30.0])
                    .with_position([64.0, 32.0]),
            )
            .with_parent(root)
            // .with_drag(true)
            .build();

        let red_frame = manager
            .add_component_with_builder()
            .with_parent(frame)
            .with_component(
                Frame::new()
                    .with_bounds([128., 45.])
                    .with_color([0.7, 0.2, 0., 1.0])
                    .with_position([0.0, 0.0]),
            )
            .with_drag_highest(true)
            .build();

        manager.add_component(
            red_frame,
            Box::new(
                Frame::new()
                    .with_bounds([32., 32.])
                    .with_color(Vec4::rgb_u32(0x277BC0))
                    .with_position([8.0, 8.0]),
            ),
        );

        let orange_frame = manager
            .add_component_with_builder()
            .with_parent(frame)
            .with_component(
                Frame::new()
                    .with_bounds([256., 128.])
                    .with_color(Vec4::rgb_u32(0xFF7F3F))
                    .with_roundness(Vec4::from([1., 1., 30., 30.]))
                    .with_edge_color([0., 0., 0., 1.0])
                    .with_position([128.0, 64.0]),
            )
            .with_drag(true)
            .build();

        for k in 0..21 {
            let row = k / 7;
            let col = k % 7;

            let color = Vec4::<f32>::rgb_u32(0x277BC0);
            manager
                .add_component_with_builder()
                .with_parent(orange_frame)
                .with_component(
                    Frame::new()
                        .with_bounds([32., 32.])
                        .with_color(color)
                        .with_roundness(Vec4::from([1., 1., 1., 1.]))
                        .with_edge_color([0., 0., 0., 1.0])
                        .with_position([10.0 + 35.0 * (col as f32), 10.0 + 33.0 * (row as f32)]),
                )
                .with_listener(GuiEventKind::OnHoverIn, |frame, _state, _e| {
                    frame.color *= 0.5;
                    frame.color[3] = 1.0;
                    None
                })
                .with_listener(GuiEventKind::OnHoverOut, |frame, _state, _e| {
                    frame.color *= 2.0;
                    frame.color[3] = 1.0;
                    None
                })
                .with_listener(GuiEventKind::OnClick, |frame, _, _| {
                    frame.color = Vec4::rgb_u32(!0);
                    None
                })
                .with_listener(GuiEventKind::OnRelease, move |frame, _, _| {
                    frame.color = color * 0.5;
                    frame.color[3] = 1.0;
                    None
                })
                .with_drag(true)
                .build();
        }
        manager
    }

    fn handle_incoming_events(&mut self) {
        self.recompute_aabb_table();
        self.process_window_events_to_generate_signals_and_queue_them_for_processing();
        self.process_signal_queue();
    }

    fn process_signal_queue(&mut self) {
        let gui_component_tree = &mut self.gui_component_tree;
        let component_signal_queue = &mut self.component_signal_queue;
        let key_to_aabb_table = &mut self.key_to_aabb_table;
        let key_to_handler_block_table = &mut self.key_to_handler_block_table;
        let program_state = self
            .state
            .as_ref()
            .expect("GuiManager state not initalized!");

        while let Some(signal) = component_signal_queue.pop_front() {
            let key = signal.component_key;
            let block = key_to_handler_block_table.get_mut(&key).unwrap();
            let event = signal.window_event_kind;
            let kind = signal.listener_kind;
            block.fire_handler(
                kind,
                EventListenerInfo {
                    event,
                    key,
                    gui_comp_tree: gui_component_tree,
                    key_to_aabb_table,
                    state: program_state,
                },
            );
        }
    }

    fn process_window_events_to_generate_signals_and_queue_them_for_processing(&mut self) {
        let window_events = &mut self.window_events;
        let key_to_aabb_table = &mut self.key_to_aabb_table;
        let gui_component_tree = &mut self.gui_component_tree;
        let focused_component = &mut self.focused_component;
        let hover_component = &mut self.hover_component;
        let component_signal_queue = &mut self.component_signal_queue;

        while let Some(event) = window_events.pop_front() {
            let _old_signal_len = component_signal_queue.len();
            match event {
                EventKind::MouseUp { .. } => {
                    if let &mut Some(gui_comp_key) = focused_component {
                        component_signal_queue.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnRelease,
                            gui_comp_key,
                            event,
                        ));
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

                    if let &mut Some(clicked) = focused_component {
                        component_signal_queue.push_front(ComponentEventSignal::new(
                            GuiEventKind::OnClick,
                            clicked,
                            event,
                        ));
                    }
                }
                EventKind::MouseMove { x, y, dx, dy } => {
                    let mouse_pos = Vec2::from([x as f32, y as f32]);
                    let _disp = Vec2::from([dx as f32, dy as f32]);
                    if let &mut Some(focused_key) = focused_component {
                        Self::object_is_focused_so_send_drag_signal_to_focused_component(
                            component_signal_queue,
                            focused_key,
                            event,
                        );
                    } else {
                        Self::check_for_hover_signal_and_send_if_found(
                            mouse_pos,
                            hover_component,
                            gui_component_tree,
                            key_to_aabb_table,
                            component_signal_queue,
                            event,
                        );
                    }
                }
                _ => (),
            }

            // if component_signal_queue.len() > _old_signal_len {
            //     println!(
            //         "signal added: {:?}",
            //         &component_signal_queue.make_contiguous()[_old_signal_len..]
            //     );
            // }
        }
    }

    fn object_is_focused_so_send_drag_signal_to_focused_component(
        component_signal_queue: &mut VecDeque<ComponentEventSignal>,
        focused_component: GuiComponentKey,
        event: EventKind,
    ) {
        component_signal_queue.push_back(ComponentEventSignal::new(
            GuiEventKind::OnDrag,
            focused_component,
            event,
        ));
    }

    fn check_for_hover_signal_and_send_if_found<'a>(
        mouse_pos: Vec2<f32>,
        hover_component: &mut Option<GuiComponentKey>,
        gui_component_tree: &'a LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
        component_signal_queue: &mut VecDeque<ComponentEventSignal>,
        event: EventKind,
    ) {
        match hover_component {
            // if something is being hovered check if mouse has left the component
            &mut Some(current_hover_key) => {
                let mut local_hover = None;

                for (key, aabb) in Self::aabb_iter(gui_component_tree, key_to_aabb_table) {
                    if aabb.is_point_inside(mouse_pos) {
                        local_hover = Some(key);
                    }
                }

                match local_hover {
                    Some(local_hover_key) => {
                        //mouse has left the current component and has entered another component
                        if local_hover_key != current_hover_key {
                            component_signal_queue.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnHoverOut,
                                current_hover_key,
                                event,
                            ));
                            component_signal_queue.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnHoverIn,
                                local_hover_key,
                                event,
                            ));
                            *hover_component = Some(local_hover_key);
                        }
                    }
                    //mouse has left the current component and is hovering over nothing
                    None => {
                        component_signal_queue.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnHoverOut,
                            current_hover_key,
                            event,
                        ));
                        //nothing is being hovered so set pointer to None
                        *hover_component = None;
                    }
                }
            }
            //if nothing is being hovered check if mouse is inside hovering a component
            None => {
                //run through aabbs in pre-order traversal
                for (key, aabb) in Self::aabb_iter(gui_component_tree, key_to_aabb_table) {
                    if aabb.is_point_inside(mouse_pos) {
                        *hover_component = Some(key);
                    }
                }
                if let &mut Some(key) = hover_component {
                    component_signal_queue.push_back(ComponentEventSignal::new(
                        GuiEventKind::OnHoverIn,
                        key,
                        event,
                    ));
                }
            }
        }
    }

    fn aabb_iter<'a>(
        gui_component_tree: &'a LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
    ) -> impl Iterator<Item = (GuiComponentKey, AABB2<f32>)> + 'a {
        gui_component_tree.iter().map(move |node_info| {
            let &key = &GuiComponentKey::from(node_info.id);
            let &aabb = key_to_aabb_table.get(&key).unwrap();
            (key, aabb)
        })
    }

    fn recompute_aabb_table(&mut self) {
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

    #[allow(dead_code)]
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
