use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    mem::MaybeUninit,
    vec,
};

use glow::HasContext;

use crate::{
    collections::{
        fixed_stack::FixedStack,
        flat_nary_tree::{LinearTree, NodeID, StackSignal},
    },
    text_writer::TextWriter,
    math::{self, translate4, ComponentWriter, Mat4, MatStack, Vec2, Vec4, AABB2},
    mem::force_borrow_mut,
    ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder, OglProg},
    window::event_util::{EventKind, KeyCode},
    FlufflState, GlowGL,
};

mod builder;
mod components;
mod gui_key;
mod handler_block;
mod renderer;

pub use self::{builder::*, components::*, gui_key::*, handler_block::*, renderer::*};
pub type ListenerCallBack<ProgramState> =
    Box<dyn FnMut(EventListenerInfo<'_, ProgramState>) -> Option<()>>;

type VisibilityStack = FixedStack<128, bool>;
type LevelStack = FixedStack<128, i32>;
type NodeStack = FixedStack<256, NodeID>;

pub type GuiMutation<T> = Box<dyn FnMut(&T)>;

pub struct MutationRequestQueue<ProgramState> {
    queue: VecDeque<GuiMutation<ProgramState>>,
}

impl<ProgramState> Default for MutationRequestQueue<ProgramState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<ProgramState> MutationRequestQueue<ProgramState> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn enqueue(&mut self, req: GuiMutation<ProgramState>) {
        self.queue.push_back(req);
    }
    pub fn dequeue(&mut self) -> Option<GuiMutation<ProgramState>> {
        self.queue.pop_front()
    }
    pub fn clear(&mut self) {
        self.queue.clear()
    }
}

pub struct GuiManager<ProgramState> {
    gl: GlowGL,

    /// lets us actually draw stuff
    renderer: GuiRenderer,

    /// used to compute global coordinates from scene-graph
    component_transform_stack: MatStack<f32>,

    /// used for cut+copy+paste
    _clipboard: String,

    ///component that is currently in "focus"
    focused_component: Option<GuiComponentKey>,

    ///component that is currently in "clicked"
    clicked_component: Option<GuiComponentKey>,

    ///component that the mouse is currently overlapping,but my not necessarily be in focus
    hover_component: Option<GuiComponentKey>,

    ///encodes the parent child relationship between nodes
    gui_component_tree: LinearTree<Box<dyn GuiComponent>>,

    /// maps a componentKey to its global AABB
    key_to_aabb_table: HashMap<GuiComponentKey, AABB2<f32>>,

    /// maps a componentKey to its event handlers
    key_to_handler_block_table: HashMap<GuiComponentKey, ComponentHandlerBlock<ProgramState>>,

    /// recomputed every Self::render(..), tells us if a component is visible (globally)
    visibility_table: Vec<bool>,

    /// used when doing visibility testing, uses this to compute cumulative intersections aabbs  
    visibility_intersection_stack: VisibilityStack,

    component_signal_bus: VecDeque<components::ComponentEventSignal>,

    key_down_table: HashSet<KeyCode>,

    window_events: VecDeque<EventKind>,

    mutation_queue: MutationRequestQueue<ProgramState>,
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn new(gl: GlowGL) -> Self {
        Self {
            renderer: GuiRenderer::new(&gl),
            focused_component: None,
            clicked_component: None,
            hover_component: None,
            gui_component_tree: LinearTree::new(),
            component_signal_bus: VecDeque::new(),
            window_events: VecDeque::new(),
            component_transform_stack: MatStack::new(),
            key_to_aabb_table: HashMap::new(),
            key_to_handler_block_table: HashMap::new(),
            key_down_table: HashSet::new(),
            visibility_table: Vec::new(),
            visibility_intersection_stack: FixedStack::new(),
            _clipboard: String::new(),
            gl,
            mutation_queue: MutationRequestQueue::new(),
        }
    }

    pub fn clear_listeners(&mut self, key: GuiComponentKey, event: GuiEventKind) {
        self.key_to_handler_block_table
            .get_mut(&key)
            .expect("key missing")
            .clear_handlers(event);
    }

    pub fn push_listener(
        &mut self,
        key: GuiComponentKey,
        listener: ComponentEventListener<ProgramState>,
    ) {
        let key_to_handler_block_table = &mut self.key_to_handler_block_table;
        key_to_handler_block_table
            .get_mut(&key)
            .expect("key missing")
            .push_handler(listener);
    }

    pub fn push_event(&mut self, event: EventKind) {
        self.window_events.push_back(event);
    }

    pub fn render(&mut self, text_writer: &mut TextWriter, window_width: f32, window_height: f32) {
        self.handle_incoming_events();

        let mut level_stack = LevelStack::new();
        let mut node_stack = NodeStack::new();

        let gl = &mut self.gl;
        let transform_stack = &mut self.component_transform_stack;

        let renderer = &mut self.renderer;
        let gui_component_tree = &mut self.gui_component_tree;
        let visibility_table = &mut self.visibility_table;
        let key_to_aabb_table = &mut self.key_to_aabb_table;

        let compute_global_position = |rel_pos, stack: &MatStack<f32>| {
            let s = stack;
            let pos = Vec4::to_pos(rel_pos);
            let &transform = s.peek();
            transform * pos
        };

        let compute_current_level = |stack: &LevelStack, comp: &dyn GuiComponent| {
            let parent_level = stack.peek();
            if comp.flags().is_set(component_flags::OVERFLOWABLE) {
                (-127, -126)
            } else {
                (parent_level, parent_level + 1)
            }
        };

        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        // forced a clone the gui_component_tree because it is actually very safe mutate
        // there are very few ways in which this tree could invalidate keys
        let gui_component_tree_borrowed_by_force =
            unsafe { crate::mem::force_borrow_mut(gui_component_tree) };

        transform_stack.clear();
        level_stack.clear_with_root_val(0);

        for (sig, key, comp) in gui_component_tree.iter_mut_stack_signals() {
            let &rel_pos = comp.rel_position();
            let transform = translate4(Vec4::to_pos(rel_pos));

            // println!("sig:{:?}",sig);
            let (cur_level, gpos) = match sig {
                StackSignal::Nop => {
                    transform_stack.pop();
                    node_stack.pop();
                    level_stack.pop();

                    let (cur_level, new_level) = compute_current_level(&level_stack, comp.as_ref());
                    let gpos = compute_global_position(rel_pos, transform_stack);

                    transform_stack.push(transform);
                    node_stack.push(key);
                    level_stack.push(new_level);

                    (cur_level, gpos)
                }
                StackSignal::Pop { n_times } => {
                    // stack.pop_multi(n_times + 1);

                    // i just said "fuck it" and popped all removable levels in one step
                    // because don't care if level is accurate for render_exit(..)
                    level_stack.pop_multi(n_times + 1);

                    for _ in 0..n_times + 1 {
                        let popped = Some(transform_stack.pop()).zip(node_stack.pop());
                        if let Some((global_frame, node)) = popped {
                            let global_position = global_frame * Vec4::from([0., 0., 0., 1.0]);
                            let visibility = visibility_table[node.as_usize()];
                            let tree =
                                unsafe { force_borrow_mut(gui_component_tree_borrowed_by_force) };
                            if visibility {
                                let state = RenderState::new(
                                    key.into(),
                                    global_position,
                                    renderer,
                                    transform_stack.len() as i32 - 1,
                                    gui_component_tree_borrowed_by_force,
                                    key_to_aabb_table,
                                    window_width,
                                    window_height,
                                );
                                tree.get_mut(node)
                                    .unwrap()
                                    .render_exit(gl, state, text_writer);
                            }
                        }
                    }

                    let (cur_level, new_level) = compute_current_level(&level_stack, comp.as_ref());
                    let gpos = compute_global_position(rel_pos, transform_stack);

                    transform_stack.push(transform);
                    node_stack.push(key);
                    level_stack.push(new_level);

                    (cur_level, gpos)
                }
                StackSignal::Push => {
                    let (cur_level, new_level) = compute_current_level(&level_stack, comp.as_ref());
                    let gpos = compute_global_position(rel_pos, transform_stack);

                    transform_stack.push(transform);
                    node_stack.push(key);
                    level_stack.push(new_level);
                    (cur_level, gpos)
                }
            };

            if visibility_table[key.as_usize()] {
                comp.render_entry(
                    gl,
                    RenderState::new(
                        key.into(),
                        gpos,
                        renderer,
                        cur_level,
                        gui_component_tree_borrowed_by_force,
                        key_to_aabb_table,
                        window_width,
                        window_height,
                    ),
                    text_writer,
                );
            }
        }

        unsafe {
            gl.disable(glow::BLEND);
        }
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    /// ## Description
    /// allows you to generate a valid Key without providing a GuiComponent up-front
    /// ## Comments
    /// - **MUST CALL `LinearTree::recontruct_preorder(..)` or the tree WONT WORK**
    unsafe fn add_component_deferred(
        &mut self,
        parent: GuiComponentKey,
        mut comp: Option<Box<dyn GuiComponent>>,
    ) -> GuiComponentKey {
        let comp = comp
            .take()
            .map(MaybeUninit::new)
            .unwrap_or(MaybeUninit::zeroed());

        let id = self
            .gui_component_tree
            .add_deferred_reconstruction(comp, parent.into());
        let key = GuiComponentKey::from(id);
        self.key_to_handler_block_table
            .insert(key, ComponentHandlerBlock::new());

        //make sure that number of nodes allocated equals
        if self.gui_component_tree.len() > self.visibility_table.len() {
            self.visibility_table.push(false)
        }

        key
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

        //make sure that number of nodes allocated equals
        if self.gui_component_tree.len() > self.visibility_table.len() {
            self.visibility_table.push(false)
        }

        key
    }

    fn handle_incoming_events(&mut self) {
        self.mutation_queue.clear();
        self.recompute_visibility();
        self.recompute_aabb_table();
        self.queue_signals_to_bus();
        self.process_signal_queue();
    }

    fn process_signal_queue(&mut self) {
        let gui_component_tree = &mut self.gui_component_tree;
        let component_signal_queue = &mut self.component_signal_bus;
        let key_to_aabb_table = &mut self.key_to_aabb_table;
        let key_to_handler_block_table = &mut self.key_to_handler_block_table;
        let mutation_queue = &mut self.mutation_queue;

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
                    mutation_queue,
                },
            );
        }
    }

    fn queue_signals_to_bus(&mut self) {
        let window_events = &mut self.window_events;
        let key_to_aabb_table = &mut self.key_to_aabb_table;
        let gui_component_tree = &mut self.gui_component_tree;
        let focused_component = &mut self.focused_component;
        let clicked_component = &mut self.clicked_component;
        let hover_component = &mut self.hover_component;
        let component_signal_bus = &mut self.component_signal_bus;
        let key_down_table = &mut self.key_down_table;
        let visibility_table = &mut self.visibility_table;
        let visibility_intersection_stack = &mut self.visibility_intersection_stack;
        let key_to_handler_block_table = &mut self.key_to_handler_block_table;

        while let Some(event) = window_events.pop_front() {
            let _old_signal_len = component_signal_bus.len();
            match event {
                EventKind::KeyDown { code } => {
                    if let KeyCode::BRACKET_R = code {
                        let v = gui_component_tree
                            .get_mut(GuiComponentKey(5))
                            .unwrap()
                            .is_visible();
                        gui_component_tree
                            .get_mut(GuiComponentKey(5))
                            .unwrap()
                            .set_visible(!v);
                    }

                    if !key_down_table.contains(&code) {
                        if let &mut Some(fkey) = focused_component {
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnKeyDown,
                                fkey,
                                event,
                            ));
                        }
                        key_down_table.insert(code);
                    }
                }
                EventKind::KeyUp { code } => {
                    if key_down_table.contains(&code) && focused_component.is_some() {
                        component_signal_bus.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnKeyRelease,
                            focused_component.expect("focused key should exist"),
                            event,
                        ));
                    }
                    key_down_table.remove(&code);
                }
                EventKind::MouseUp { .. } => {
                    if let &mut Some(gui_comp_key) = clicked_component {
                        component_signal_bus.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnMouseRelease,
                            gui_comp_key,
                            event,
                        ));
                    }

                    *clicked_component = None;
                }
                EventKind::MouseDown { x, y, .. } => {
                    let mouse_pos = Vec2::from([x as f32, y as f32]);
                    let prev_focused_component = *focused_component;

                    *clicked_component = None;
                    *focused_component = None;

                    Self::point_in_aabb_cumulative_intersections(
                        gui_component_tree,
                        key_to_aabb_table,
                        visibility_table,
                        visibility_intersection_stack,
                        mouse_pos,
                        |key| {
                            *clicked_component = Some(key);
                            *focused_component = Some(key);
                        },
                    );

                    // handle focused events
                    match (prev_focused_component, *focused_component) {
                        (None, Some(cur_key)) => {
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnFocusIn,
                                cur_key,
                                event,
                            ));
                        }
                        (Some(prev_key), None) => {
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnFocusOut,
                                prev_key,
                                event,
                            ));
                        }
                        (Some(prev_key), Some(cur_key)) => {
                            if prev_key != cur_key {
                                component_signal_bus.push_back(ComponentEventSignal::new(
                                    GuiEventKind::OnFocusOut,
                                    prev_key,
                                    event,
                                ));
                                component_signal_bus.push_back(ComponentEventSignal::new(
                                    GuiEventKind::OnFocusIn,
                                    cur_key,
                                    event,
                                ));
                            }
                        }

                        (None, None) => {
                            // do nothing
                        }
                    }

                    if let &mut Some(clicked) = clicked_component {
                        component_signal_bus.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnMouseDown,
                            clicked,
                            event,
                        ));
                    }
                }
                EventKind::MouseMove { x, y, dx, dy } => {
                    let mouse_pos = Vec2::from([x as f32, y as f32]);
                    let _disp = Vec2::from([dx as f32, dy as f32]);

                    if let &mut Some(hover_key) = hover_component {
                        if visibility_table[hover_key] {
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnMouseMove,
                                hover_key,
                                event,
                            ));
                        }
                    }

                    if let &mut Some(clicked_key) = clicked_component {
                        //force release of component if its being clicked on while being invisible
                        if !visibility_table[clicked_key] && clicked_component.is_some() {
                            let clicked_key = clicked_component.expect("clicked should be valid");
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnMouseRelease,
                                clicked_key,
                                event,
                            ));
                            *clicked_component = None;
                        }

                        Self::object_is_clicked_so_send_drag_signal_to_focused_component(
                            gui_component_tree,
                            component_signal_bus,
                            key_to_handler_block_table,
                            clicked_key,
                            event,
                        );
                    } else {
                        Self::check_for_hover_signal_and_send_if_found(
                            mouse_pos,
                            hover_component,
                            gui_component_tree,
                            key_to_aabb_table,
                            component_signal_bus,
                            visibility_table,
                            visibility_intersection_stack,
                            event,
                        );
                    }
                }
                EventKind::MouseWheel { .. } => {
                    if let &mut Some(focused_key) = focused_component {
                        component_signal_bus.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnWheelWhileFocused,
                            focused_key,
                            event,
                        ));
                    }

                    if let &mut Some(hovered_key) = hover_component {
                        Self::push_signal_to_bus_and_bubble(
                            component_signal_bus,
                            gui_component_tree,
                            key_to_handler_block_table,
                            hovered_key,
                            GuiEventKind::OnWheelWhileHovered,
                            event,
                        );
                    }
                }
                _ => (),
            }

            Self::print_signals_queued_to_bus(component_signal_bus, _old_signal_len);
        }
    }

    fn print_signals_queued_to_bus(
        _component_signal_bus: &mut VecDeque<ComponentEventSignal>,
        _old_signal_len: usize,
    ) {
        // prints the queued events that waiting to be sent to their handlers
        // if _component_signal_bus.len() > _old_signal_len {
        //     println!("signal added:");
        //     for sig in &_component_signal_bus.make_contiguous()[_old_signal_len..] {
        //         println!("{:?}", sig)
        //     }
        // }
    }

    fn object_is_clicked_so_send_drag_signal_to_focused_component(
        gui_component_tree: &LinearTree<Box<dyn GuiComponent>>,
        component_signal_bus: &mut VecDeque<ComponentEventSignal>,
        key_to_handler_block_table: &HashMap<GuiComponentKey, ComponentHandlerBlock<ProgramState>>,
        clicked_component: GuiComponentKey,
        event: EventKind,
    ) {
        Self::push_signal_to_bus_and_bubble(
            component_signal_bus,
            gui_component_tree,
            key_to_handler_block_table,
            clicked_component,
            GuiEventKind::OnDrag,
            event,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn check_for_hover_signal_and_send_if_found<'a>(
        mouse_pos: Vec2<f32>,
        hover_component: &mut Option<GuiComponentKey>,
        gui_component_tree: &'a LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
        component_signal_bus: &mut VecDeque<ComponentEventSignal>,
        visibility_table: &'a [bool],
        visibility_intersection_stack: &'a mut FixedStack<128, bool>,
        event: EventKind,
    ) {
        match hover_component {
            // if something is being hovered check if mouse has left the component
            &mut Some(current_hover_key) => {
                let mut local_hover = None;

                Self::point_in_aabb_cumulative_intersections(
                    gui_component_tree,
                    key_to_aabb_table,
                    visibility_table,
                    visibility_intersection_stack,
                    mouse_pos,
                    |key| {
                        local_hover = Some(key);
                    },
                );

                match local_hover {
                    Some(local_hover_key) => {
                        //mouse has left the current component and has entered another component
                        if local_hover_key != current_hover_key {
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnHoverOut,
                                current_hover_key,
                                event,
                            ));
                            component_signal_bus.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnHoverIn,
                                local_hover_key,
                                event,
                            ));
                            *hover_component = Some(local_hover_key);
                        }
                    }
                    //mouse has left the current component and is hovering over nothing
                    None => {
                        component_signal_bus.push_back(ComponentEventSignal::new(
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
                Self::point_in_aabb_cumulative_intersections(
                    gui_component_tree,
                    key_to_aabb_table,
                    visibility_table,
                    visibility_intersection_stack,
                    mouse_pos,
                    |key| {
                        *hover_component = Some(key);
                    },
                );

                if let &mut Some(key) = hover_component {
                    component_signal_bus.push_back(ComponentEventSignal::new(
                        GuiEventKind::OnHoverIn,
                        key,
                        event,
                    ));
                }
            }
        }
    }

    fn recompute_visibility(&mut self) {
        let gui_component_tree = &mut self.gui_component_tree;
        let get_visibility = |id| gui_component_tree.get(id).map(|state| state.is_visible());
        for node in gui_component_tree.iter() {
            let mut cur_node_id = node.id;
            self.visibility_table[cur_node_id.as_usize()] = get_visibility(node.id).unwrap();
            while let Some(parent) = gui_component_tree.get_parent_id(cur_node_id) {
                if !get_visibility(parent).unwrap() {
                    self.visibility_table[node.id.as_usize()] = false;
                    break;
                }
                cur_node_id = parent;
            }
        }
    }

    fn point_in_aabb_cumulative_intersections<'a, CB>(
        gui_component_tree: &'a LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
        visibility_table: &'a [bool],
        visibility_stack: &'a mut VisibilityStack,
        mouse_pos: Vec2<f32>,
        mut callback: CB,
    ) where
        CB: FnMut(GuiComponentKey),
    {
        visibility_stack.clear_with_root_val(true);

        let node_iter = gui_component_tree
            .iter_stack_signals()
            .map(|(sig, id, c)| (sig, GuiComponentKey::from(id), c));

        for (sig, key, component) in node_iter {
            let &aabb = key_to_aabb_table.get(&key).unwrap();
            let is_mouse_inside = aabb.is_point_inside(mouse_pos) || component.is_origin();
            let is_overflowable = component.is_overflowable();

            let calc_intersected_visibility =
                |current_visibility| (current_visibility || is_overflowable) && is_mouse_inside;

            let intersected_visibility = match sig {
                StackSignal::Nop => {
                    visibility_stack.pop();
                    let current_visibility = visibility_stack.peek();
                    let intersected_visibility = calc_intersected_visibility(current_visibility);
                    visibility_stack.push(intersected_visibility);
                    intersected_visibility
                }
                StackSignal::Pop { n_times } => {
                    visibility_stack.pop_multi(n_times + 1);
                    let current_visibility = visibility_stack.peek();
                    let intersected_visibility = calc_intersected_visibility(current_visibility);
                    visibility_stack.push(intersected_visibility);
                    intersected_visibility
                }
                StackSignal::Push => {
                    let current_visibility = visibility_stack.peek();
                    let intersected_visibility = calc_intersected_visibility(current_visibility);
                    visibility_stack.push(intersected_visibility);
                    intersected_visibility
                }
            };

            if intersected_visibility && visibility_table[key.as_usize()] {
                callback(key);
            }
        }
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

    /// pushes the signal onto the bus headed for `key` but if `key` has no handler for `sig_kind`
    /// it will redirect signal to an ancestor of `key`
    fn push_signal_to_bus_and_bubble(
        component_signal_bus: &mut VecDeque<ComponentEventSignal>,
        gui_component_tree: &LinearTree<Box<dyn GuiComponent>>,
        key_to_handler_block_table: &HashMap<GuiComponentKey, ComponentHandlerBlock<ProgramState>>,
        key: GuiComponentKey,
        sig_kind: GuiEventKind,
        event: EventKind,
    ) {
        let current_node_has_listener = key_to_handler_block_table
            .get(&key)
            .and_then(|block| block.get(sig_kind as usize))
            .map(|wheel_hovered_handlers| !wheel_hovered_handlers.is_empty())
            .unwrap_or(false);

        if current_node_has_listener {
            component_signal_bus.push_back(ComponentEventSignal::new(sig_kind, key, event));
        } else {
            let parent_query = Self::find_ancestor_with_event_handler_kind(
                key,
                sig_kind,
                gui_component_tree,
                key_to_handler_block_table,
            );
            if let Some(parent) = parent_query {
                component_signal_bus.push_back(ComponentEventSignal::new(sig_kind, parent, event));
            }
        }
    }

    fn find_ancestor_with_event_handler_kind(
        root: GuiComponentKey,
        target_event_kind: GuiEventKind,
        gui_component_tree: &LinearTree<Box<dyn GuiComponent>>,
        key_to_handler_block_table: &HashMap<GuiComponentKey, ComponentHandlerBlock<ProgramState>>,
    ) -> Option<GuiComponentKey> {
        let mut node = root;

        while let Some(parent) = gui_component_tree.get_parent_id(node) {
            let number_of_listeners_for_target_event = key_to_handler_block_table
                .get(&parent.into())
                .expect("listener block not found")
                .get(target_event_kind as usize)
                .expect("block not initalized")
                .len();

            if number_of_listeners_for_target_event > 0 {
                return Some(parent.into());
            }
            node = parent.into();
        }

        None
    }

    pub fn poll_mutation_requsts(&mut self) -> Option<GuiMutation<ProgramState>> {
        self.mutation_queue.dequeue()
    }
}

impl<State> GuiManager<FlufflState<State>> {
    /// ## Description
    /// Certain event handlers may make requests to mutate `ProgramState` and this function
    /// is responsible for executing those requests.
    /// ## Comments
    /// - Works be dequeing the `mutation_queue`
    /// - `FluffleState<State>` is smartpointer with a read/write lock, so we need to access the mutation request
    /// queue carefully. This pointer is expected to point to a memory location that owns a GuiManager `Self`.
    pub fn execute_mutation_requests<CB>(state: &FlufflState<State>, mut get_request: CB)
    where
        CB: FnMut(&FlufflState<State>) -> Option<GuiMutation<FlufflState<State>>>,
    {
        while let Some(mut req) = get_request(state) {
            req(state);
        }
    }
}
