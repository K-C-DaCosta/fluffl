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
    extras::{
        ogl::{self, ArrayBuilder, Bindable, BufferPair, HasBufferBuilder, OglProg},
        text_writer::TextWriter,
    },
    math::{self, stack::MatStack, translate4, ComponentWriter, Mat4, Vec2, Vec4, AABB2},
    mem::force_borrow_mut,
    window::event_util::{EventKind, KeyCode},
    GlowGL,
};

mod builder;
mod components;
mod gui_key;
mod handler_block;
mod renderer;

use self::{builder::*, components::*, gui_key::*, handler_block::*, renderer::*};

pub type ListenerCallBack<ProgramState> =
    Box<dyn FnMut(EventListenerInfo<'_, ProgramState>) -> Option<()>>;

type VisibilityStack = FixedStack<128, bool>;

pub struct GuiManager<ProgramState> {
    gl: GlowGL,

    state: Option<ProgramState>,

    /// lets us actually draw stuff
    renderer: GuiRenderer,

    /// used to compute global coordinates from scene-graph
    component_transform_stack: MatStack<f32>,

    /// used for cut+copy+paste
    clipboard: String,

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

    component_signal_queue: VecDeque<components::ComponentEventSignal>,

    key_down_table: HashSet<KeyCode>,

    window_events: VecDeque<EventKind>,
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn new(gl: GlowGL) -> Self {
        Self {
            renderer: GuiRenderer::new(&gl),
            focused_component: None,
            clicked_component: None,
            hover_component: None,
            gui_component_tree: LinearTree::new(),
            component_signal_queue: VecDeque::new(),
            window_events: VecDeque::new(),
            component_transform_stack: MatStack::new(),
            key_to_aabb_table: HashMap::new(),
            key_to_handler_block_table: HashMap::new(),
            key_down_table: HashSet::new(),
            visibility_table: Vec::new(),
            visibility_intersection_stack: FixedStack::new(),
            clipboard: String::new(),
            gl,
            state: None,
        }
        .setup_test_gui()
    }

    pub fn init_state(&mut self, state: ProgramState) {
        self.state = Some(state);
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

        let gl = &mut self.gl;
        let stack = &mut self.component_transform_stack;

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
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        // forced a clone the gui_component_tree because it is actually very safe mutate
        // there are very few ways in which this tree could invalidate keys
        let gui_component_tree_borrowed_by_force =
            unsafe { crate::mem::force_borrow_mut(gui_component_tree) };

        stack.clear();
        for (sig, key, comp) in gui_component_tree.iter_mut_stack_signals() {
            let &rel_pos = comp.rel_position();
            let transform = translate4(Vec4::to_pos(rel_pos));

            // println!("sig:{:?}",sig);
            let gpos = match sig {
                StackSignal::Nop => {
                    stack.pop();
                    let gpos = compute_global_position(rel_pos, stack);
                    stack.push(transform);
                    gpos
                }
                StackSignal::Pop { n_times } => {
                    stack.pop_multi(n_times + 1);
                    let gpos = compute_global_position(rel_pos, stack);
                    stack.push(transform);
                    gpos
                }
                StackSignal::Push => {
                    let gpos = compute_global_position(rel_pos, stack);
                    stack.push(transform);
                    gpos
                }
            };

            if visibility_table[key.as_usize()] {
                comp.render(
                    gl,
                    RenderState::new(
                        key.into(),
                        gpos,
                        renderer,
                        stack.len() - 1,
                        gui_component_tree_borrowed_by_force,
                        key_to_aabb_table,
                    ),
                    text_writer,
                    window_width,
                    window_height,
                );
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
        let origin =
            manager.add_component(GuiComponentKey::default(), Box::new(OriginState::new()));

        let prink_frame = manager
            .builder_frame()
            .with_parent(origin)
            .with_bounds([400.0 + 0.0, 200.0 + 100.0])
            .with_roundness([1., 1., 1.0, 1.0])
            .with_position([64.0, 32.0])
            .with_scrollbars(true)
            // .with_drag(true)
            .build();

        let alt_frame = manager
            .builder_frame()
            .with_parent(origin)
            .with_bounds([200.0, 100.0])
            .with_roundness([0.0, 0.0, 10.0, 10.])
            .with_position([64.0, 400.0])
            .with_drag(true)
            .build();

        let red_frame = manager
            .builder_frame()
            .with_parent(prink_frame)
            .with_bounds([400., 45.])
            .with_color([0.7, 0.2, 0., 1.0])
            .with_position([0.0, 0.0])
            .with_drag_highest(true)
            .build();

        let red_child = manager
            .builder_frame()
            .with_parent(red_frame)
            .with_bounds([32., 32.])
            .with_color(Vec4::rgb_u32(0x277BC0))
            .with_position([8.0, 8.0])
            .build();

        let orange_frame = manager
            .builder_frame()
            .with_parent(prink_frame)
            .with_bounds([256., 128.])
            .with_color(Vec4::rgb_u32(0xFF7F3F))
            .with_roundness(Vec4::from([1., 1., 30., 30.]))
            .with_edge_color([0., 0., 0., 1.0])
            .with_position([128.0, 64.0])
            .with_drag(true)
            .with_visibility(false)
            .build();

        let slider_frame = manager
            .builder_slider()
            .with_parent(prink_frame)
            .with_position([4.0, 64.0])
            .with_bounds([128.0, 32.0])
            .with_color(Vec4::rgb_u32(0x554994))
            .with_edge_color(Vec4::rgb_u32(0xFFCCB3))
            .with_roundness([8.0; 4])
            .with_drag(false)
            .with_listener(GuiEventKind::OnFocusIn, |state, _, _| {
                state.slider_frame.edge_color = Vec4::rgb_u32(0xff0000);
            })
            .with_listener(GuiEventKind::OnFocusOut, |state, _, _| {
                state.slider_frame.edge_color = Vec4::rgb_u32(0xFFCCB3);
            })
            .with_button_bounds([32.0, 120.0])
            .with_button_color(Vec4::rgb_u32(0x332255))
            .with_button_edge_color(Vec4::rgb_u32(0xF29393))
            .with_button_roundness([8.0; 4])
            .with_button_listener(GuiEventKind::OnHoverIn, |f, _, _| {
                f.color *= 9. / 10.;
            })
            .with_button_listener(GuiEventKind::OnHoverOut, |f, _, _| {
                f.color *= 10. / 9.;
            })
            .with_button_listener(GuiEventKind::OnMouseDown, |f, _, _| {
                f.color = Vec4::from([1.0; 4]) - f.color;
            })
            .with_button_listener(GuiEventKind::OnMouseRelease, |f, _, _| {
                f.color = Vec4::from([1.0; 4]) - f.color;
            })
            .build();

        for k in 0..1 {
            let row = k / 7;
            let col = k % 7;
            let color = Vec4::<f32>::rgb_u32(0x277BC0);
            let _blue_button = manager
                .builder_frame()
                .with_parent(orange_frame)
                .with_bounds([32., 32.])
                .with_color(color)
                .with_roundness(Vec4::from([1., 1., 1., 1.]))
                .with_edge_color([0., 0., 0., 1.0])
                .with_position([7.0 + 35.0 * (col as f32), 5.0 + 33.0 * (row as f32)])
                .with_listener(GuiEventKind::OnHoverIn, |frame, _state, _e| {
                    frame.color *= 0.5;
                    frame.color[3] = 1.0;
                })
                .with_listener(GuiEventKind::OnHoverOut, |frame, _state, _e| {
                    frame.color *= 2.0;
                    frame.color[3] = 1.0;
                })
                .with_listener(GuiEventKind::OnMouseDown, |frame, _, _| {
                    frame.color = Vec4::rgb_u32(!0);
                })
                .with_listener(GuiEventKind::OnMouseRelease, move |frame, _, _| {
                    frame.color = color * 0.5;
                    frame.color[3] = 1.0;
                })
                .with_drag(true)
                .build();
        }

        let textbox_key = manager
            .builder_textbox()
            .with_parent(prink_frame)
            .with_bounds([1000.0, 64.0])
            .with_position([4.0, 200.0 - 64.0])
            .with_color(Vec4::rgb_u32(0))
            .with_roundness([0.0, 0.0, 32.0, 32.0])
            .with_font_size(32.0)
            .with_alignment([TextAlignment::Left, TextAlignment::Center])
            .with_listener(GuiEventKind::OnFocusIn, |comp, _, _| {
                comp.frame.edge_color = Vec4::rgb_u32(0xff0000);
            })
            .with_listener(GuiEventKind::OnFocusOut, |comp, _, _| {
                comp.frame.edge_color = Vec4::rgb_u32(0x89CFFD);
            })
            .build();

        // println!("origin={}", origin);
        // println!("pink_frame={}", prink_frame);
        // println!("orange_frame={}", orange_frame);
        // println!("blue_button={}", blue_button);
        // println!("slider_frame={}", slider_frame);
        // println!("slider_button={}", slider_button);
        manager.gui_component_tree.print_by_ids();
        // let parent = manager.gui_component_tree.get_parent_id(NodeID(4)).unwrap();
        // println!("parent of 4 is = {:?}", parent);
        manager
    }

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
            .map(|v| MaybeUninit::new(v))
            .unwrap_or(MaybeUninit::zeroed());

        let id = self
            .gui_component_tree
            .add_deffered_reconstruction(comp, parent.into());
        let key = GuiComponentKey::from(id);
        self.key_to_handler_block_table
            .insert(key, ComponentHandlerBlock::new());

        //make sure that number of nodes allocated equals
        if self.gui_component_tree.len() > self.visibility_table.len() {
            self.visibility_table.push(false)
        }

        key
    }

    fn add_component(
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
        self.recompute_visibility();
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
        let clicked_component = &mut self.clicked_component;
        let hover_component = &mut self.hover_component;
        let component_signal_queue = &mut self.component_signal_queue;
        let key_down_table = &mut self.key_down_table;
        let visibility_table = &mut self.visibility_table;
        let visibility_intersection_stack = &mut self.visibility_intersection_stack;

        while let Some(event) = window_events.pop_front() {
            let _old_signal_len = component_signal_queue.len();
            match event {
                EventKind::KeyDown { code } => {
                    if let KeyCode::BRAKET_RIGHT = code {
                        let v = gui_component_tree
                            .get_mut(GuiComponentKey(5))
                            .unwrap()
                            .is_visible();
                        gui_component_tree
                            .get_mut(GuiComponentKey(5))
                            .unwrap()
                            .set_visible(!v);
                    }

                    if key_down_table.contains(&code) == false {
                        if let &mut Some(fkey) = focused_component {
                            component_signal_queue.push_back(ComponentEventSignal::new(
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
                        component_signal_queue.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnKeyRelease,
                            focused_component.expect("focused key should exist"),
                            event,
                        ));
                    }
                    key_down_table.remove(&code);
                }
                EventKind::MouseUp { .. } => {
                    if let &mut Some(gui_comp_key) = clicked_component {
                        component_signal_queue.push_back(ComponentEventSignal::new(
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
                            component_signal_queue.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnFocusIn,
                                cur_key,
                                event,
                            ));
                        }
                        (Some(prev_key), None) => {
                            component_signal_queue.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnFocusOut,
                                prev_key,
                                event,
                            ));
                        }
                        (Some(prev_key), Some(cur_key)) => {
                            if prev_key != cur_key {
                                component_signal_queue.push_back(ComponentEventSignal::new(
                                    GuiEventKind::OnFocusOut,
                                    prev_key,
                                    event,
                                ));
                                component_signal_queue.push_back(ComponentEventSignal::new(
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
                        component_signal_queue.push_back(ComponentEventSignal::new(
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
                            component_signal_queue.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnMouseMove,
                                hover_key,
                                event,
                            ));
                        }
                    }

                    if let &mut Some(clicked_key) = clicked_component {
                        //force release of component if its being clicked on while being invisible
                        if visibility_table[clicked_key] == false && clicked_component.is_some() {
                            let clicked_key = clicked_component.expect("clicked should be valid");
                            component_signal_queue.push_back(ComponentEventSignal::new(
                                GuiEventKind::OnMouseRelease,
                                clicked_key,
                                event,
                            ));
                            *clicked_component = None;
                        }

                        Self::object_is_clicked_so_send_drag_signal_to_focused_component(
                            component_signal_queue,
                            clicked_key,
                            event,
                        );
                    } else {
                        Self::check_for_hover_signal_and_send_if_found(
                            mouse_pos,
                            hover_component,
                            gui_component_tree,
                            key_to_aabb_table,
                            component_signal_queue,
                            visibility_table,
                            visibility_intersection_stack,
                            event,
                        );
                    }
                }
                EventKind::MouseWheel { .. } => {
                    if let &mut Some(focused_key) = focused_component {
                        component_signal_queue.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnWheelWhileFocused,
                            focused_key,
                            event,
                        ));
                    }
                    if let &mut Some(hovered_key) = hover_component {
                        component_signal_queue.push_back(ComponentEventSignal::new(
                            GuiEventKind::OnWheelWhileHovered,
                            hovered_key,
                            event,
                        ));
                    }
                }
                _ => (),
            }

            // // prints the queued events that waiting to be sent to their handlers
            // if component_signal_queue.len() > _old_signal_len {
            //     println!("signal added:");
            //     for sig in &component_signal_queue.make_contiguous()[_old_signal_len..] {
            //         println!("{:?}", sig)
            //     }
            // }
        }
    }

    fn object_is_clicked_so_send_drag_signal_to_focused_component(
        component_signal_queue: &mut VecDeque<ComponentEventSignal>,
        clicked_component: GuiComponentKey,
        event: EventKind,
    ) {
        component_signal_queue.push_back(ComponentEventSignal::new(
            GuiEventKind::OnDrag,
            clicked_component,
            event,
        ));
    }

    fn check_for_hover_signal_and_send_if_found<'a>(
        mouse_pos: Vec2<f32>,
        hover_component: &mut Option<GuiComponentKey>,
        gui_component_tree: &'a LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
        component_signal_queue: &mut VecDeque<ComponentEventSignal>,
        visibility_table: &'a Vec<bool>,
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
                    component_signal_queue.push_back(ComponentEventSignal::new(
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
                if get_visibility(parent).unwrap() == false {
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
        visibility_table: &'a Vec<bool>,
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

        for (sig, key, c) in node_iter {
            let &aabb = key_to_aabb_table.get(&key).unwrap();
            let is_mouse_inside = aabb.is_point_inside(mouse_pos) || c.is_origin();

            let intersected_visibility = match sig {
                StackSignal::Nop => {
                    visibility_stack.pop();
                    let current_visibility = visibility_stack.peek();
                    let intersected_visibility = current_visibility && is_mouse_inside;
                    visibility_stack.push(intersected_visibility);
                    intersected_visibility
                }
                StackSignal::Pop { n_times } => {
                    visibility_stack.pop_multi(n_times + 1);
                    let current_visibility = visibility_stack.peek();
                    let intersected_visibility = current_visibility && is_mouse_inside;
                    visibility_stack.push(intersected_visibility);
                    intersected_visibility
                }
                StackSignal::Push => {
                    let current_visibility = visibility_stack.peek();
                    let intersected_visibility = current_visibility && is_mouse_inside;
                    visibility_stack.push(intersected_visibility);
                    intersected_visibility
                }
            };

            if intersected_visibility && visibility_table[key] {
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
}
