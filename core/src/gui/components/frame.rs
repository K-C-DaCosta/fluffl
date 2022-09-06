use super::*;
use crate::collections::flat_nary_tree::NodeInfoMut;

#[derive(Copy, Clone)]
/// at this point this is basically a 3x3 basis 
pub struct SliderRail {
    min: Vec2<f32>,
    horizontal_disp: Vec2<f32>,
    vertical_disp: Vec2<f32>,
}
impl SliderRail {
    pub fn eval(&self, u: f32, v: f32) -> Vec2<f32> {
        self.min + self.horizontal_disp * u + self.vertical_disp * v
    }
}

#[derive(Clone)]
pub struct FrameState {
    pub bounds: Vec2<f32>,
    pub rel_pos: Vec2<f32>,
    pub color: Vec4<f32>,
    pub edge_color: Vec4<f32>,
    pub roundness: Vec4<f32>,
    pub is_visible: bool,
    is_scrollbars_enabled: bool,

    camera: Vec2<f32>,
    rails: Option<SliderRail>,
    percentages: Vec2<f32>,
    components_aabb: AABB2<f32>,
    horizontal_scroll_area: AABB2<f32>,
    vertical_scroll_area: AABB2<f32>,
    last_known_mouse_pos: Vec2<f32>,
}

impl FrameState {
    pub fn new() -> Self {
        Self {
            bounds: Vec2::from([128.; 2]),
            rel_pos: Vec2::from([0.0; 2]),
            color: Vec4::rgb_u32(0xF94892),
            edge_color: Vec4::rgb_u32(0x89CFFD),
            roundness: Vec4::from([1.0, 1.0, 1.0, 1.0]),
            is_visible: true,
            is_scrollbars_enabled: false,
            camera: Vec2::zero(),
            components_aabb: AABB2::zero(),
            horizontal_scroll_area: AABB2::zero(),
            vertical_scroll_area: AABB2::zero(),
            rails: None,
            percentages: Vec2::zero(),
            last_known_mouse_pos: Vec2::zero(),
        }
    }

    fn draw_rectangle(gl: &GlowGL, r: &GuiRenderer, win_w: f32, win_h: f32, rect: AABB2<f32>) {
        unsafe {
            gl.blend_func(glow::ONE, glow::ONE);
        }
        r.builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_roundness_vec([1.0; 4])
            .set_edge_color(Vec4::rgb_u32(!0))
            .set_background_color(Vec4::rgb_u32(0))
            .set_bounds(rect.dims())
            .set_position(Vec4::convert(rect.min_pos), Vec4::convert(rect.dims()))
            .render();

        unsafe {
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
    }

    fn update_component_bounds(
        &mut self,
        gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
        root_key: GuiComponentKey,
    ) {
        if self.rails.is_some() {
            return;
        }
        self.components_aabb = compute_component_bounds(gui_component_tree, root_key);
        self.update_component_bounds_assuming_new_bounds_already_set();
    }

    fn update_component_bounds_assuming_new_bounds_already_set(&mut self) {
        if self.rails.is_some() {
            return;
        }
        // overflow vectors for the "right" and "down" half-spaces of the frame
        // we don't care about overflow on the "left" and "top" half-spaces
        let horizontal_disp = Vec2::from([
            (self.bounds.x() - self.components_aabb.max_pos.x()).min(0.0),
            0.0,
        ]);
        let vertical_disp = Vec2::from([
            0.0,
            (self.bounds.y() - self.components_aabb.max_pos.y()).min(0.0),
        ]);

        self.rails = Some(SliderRail {
            min: self.components_aabb.min_pos,
            vertical_disp,
            horizontal_disp,
        });
    }
}

impl GuiComponent for FrameState {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn is_visible(&self) -> bool {
        self.is_visible
    }
    fn set_visible(&mut self, is_visible: bool) {
        self.is_visible = is_visible;
    }

    fn get_bounds(&self) -> Vec2<f32> {
        self.bounds
    }
    fn set_bounds(&mut self, bounds: Vec2<f32>) {
        self.bounds = bounds;
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.rel_pos = pos;
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.rel_pos
    }

    fn render<'b>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'b>,
        _text_writer: &mut TextWriter,
        win_w: f32,
        win_h: f32,
    ) {
        if self.is_visible == false {
            return;
        }

        let r = state.renderer;
        let level = state.level;
        let pos = Vec2::convert(state.global_position);

        layer_lock(gl, level);

        r.builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_roundness_vec(self.roundness)
            .set_edge_color(self.edge_color)
            .set_background_color(self.color)
            .set_null_color([0., 0., 0., 0.0])
            .set_bounds(self.bounds)
            .set_position(state.global_position, Vec4::to_pos(self.bounds))
            .render();
        layer_unlock(gl);

        //compute global horizontal bounding box
        const HORIZONTAL_SCROLL_HEIGHT: f32 = 20.0;
        self.horizontal_scroll_area = AABB2::from_point_and_lengths(
            Vec2::from([
                pos.x(),
                pos.y() + self.bounds.y() - HORIZONTAL_SCROLL_HEIGHT,
            ]),
            Vec2::from([self.bounds.x(), HORIZONTAL_SCROLL_HEIGHT]),
        );
        if self.is_scrollbars_enabled {
            Self::draw_rectangle(gl, r, win_w, win_h, self.horizontal_scroll_area);

            let s0 = self.horizontal_scroll_area.min_pos;
            let u = self.percentages.x();

            let cursor_bounds = Vec2::from([15.0, HORIZONTAL_SCROLL_HEIGHT]);
            let cursor_pos =
                Vec2::from([(self.bounds.x() - cursor_bounds.x()) * u + s0.x(), s0.y()]);

            let cursor_aabb = AABB2::from_point_and_lengths(cursor_pos, cursor_bounds);
            Self::draw_rectangle(gl, r, win_w, win_h, cursor_aabb);
        }
    }
}

pub fn compute_alignment_position(
    global_position: Vec2<f32>,
    text_bounds: Vec2<f32>,
    component_bounds: Vec2<f32>,
    alignment: &[TextAlignment; 2],
) -> Vec2<f32> {
    let mut res = Vec2::zero();
    for pos_idx in 0..res.len() {
        let comp_gpos = global_position[pos_idx];
        let comp_dim = component_bounds[pos_idx];
        let text_dim = text_bounds[pos_idx];
        let alignment_mode = alignment[pos_idx];
        res[pos_idx] = match alignment_mode {
            TextAlignment::Left | TextAlignment::Stretch => comp_gpos,
            TextAlignment::Right => comp_gpos + comp_dim - text_dim,
            TextAlignment::Center => comp_gpos + (comp_dim - text_dim) * 0.5,
        };
    }
    res
}

pub struct FrameBuilder<'a, ProgramState> {
    manager: &'a mut GuiManager<ProgramState>,
    state: Option<FrameState>,
    parent: Option<GuiComponentKey>,
    frame_key: Option<GuiComponentKey>,
}

impl<'a, ProgramState> FrameBuilder<'a, ProgramState> {
    pub fn new(manager: &'a mut GuiManager<ProgramState>) -> Self {
        let frame_key =
            unsafe { Some(manager.add_component_deferred(GuiComponentKey::default(), None)) };
        Self {
            manager,
            state: Some(FrameState::new()),
            parent: None,
            frame_key,
        }
    }

    pub fn with_bounds<T>(mut self, bounds: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        let bounds = Vec2::from(bounds);
        self.state.as_mut().unwrap().bounds = bounds;
        self
    }

    pub fn with_color<T>(mut self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.state.as_mut().unwrap().color = Vec4::from(color);
        self
    }

    pub fn with_edge_color<T>(mut self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.state.as_mut().unwrap().edge_color = Vec4::from(color);
        self
    }

    pub fn with_roundness<T>(mut self, r: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.state.as_mut().unwrap().roundness = Vec4::from(r);
        self
    }

    pub fn with_position<T>(mut self, pos: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        self.state.as_mut().unwrap().rel_pos = Vec2::from(pos);
        self
    }

    pub fn with_scrollbars(mut self, enable: bool) -> Self {
        if enable {
            self.state.as_mut().unwrap().is_scrollbars_enabled = true;

            self.with_listener_advanced(
                GuiEventKind::OnDrag,
                Box::new(|info| {
                    let root_key = info.key;
                    let gui_component_tree = info.gui_comp_tree;
                    let mouse_pos = info.event.mouse_pos();

                    let frame = get_frame(
                        unsafe { crate::mem::force_borrow_mut(gui_component_tree) },
                        root_key,
                    );
                    frame.update_component_bounds(gui_component_tree, root_key);

                    let new_component_bounding_box =
                        compute_component_bounds(gui_component_tree, root_key);
                    let old_component_bounding_box = frame.components_aabb;
                    const EPSILON: f32 = 0.001;

                    let component_bounding_box_changed_dramatically = new_component_bounding_box
                        .dims()
                        .iter()
                        .zip(old_component_bounding_box.dims().iter())
                        .all(|(&cur, &prev)| (cur - prev).abs() < EPSILON)
                        == false;

                    if component_bounding_box_changed_dramatically {
                        let old_uv = frame.percentages;
                        scroll_elements(gui_component_tree, root_key, Vec2::zero());
                        frame.rails = None;
                        frame.update_component_bounds(gui_component_tree, root_key);
                        scroll_elements(gui_component_tree, root_key, old_uv);
                    }

                    let uv = {
                        let frame = get_frame(gui_component_tree, root_key);
                        let frame_pos = info.key_to_aabb_table.get(&root_key).unwrap().min_pos;
                        let frame_bounds = frame.bounds;
                        let mut uv = (mouse_pos - frame_pos) / frame_bounds;
                        uv.iter_mut().for_each(|comp| *comp = comp.clamp(0.0, 1.0));
                        uv
                    };

                    let (horizontal_scroll_area, _vertical_scroll_area) = {
                        let frame = get_frame(gui_component_tree, root_key);
                        let hsa = frame.horizontal_scroll_area;
                        let vsa = frame.vertical_scroll_area;
                        (hsa, vsa)
                    };

                    if horizontal_scroll_area.is_point_inside(mouse_pos) {
                        frame.percentages[0] = uv.x();
                        let uv = frame.percentages;
                        scroll_elements(gui_component_tree, root_key, uv);
                    }
                    None
                }),
            )
            .with_listener_advanced(
                GuiEventKind::OnWheelWhileHovered,
                Box::new(|info| {
                    let root_key = info.key;
                    let gui_component_tree = info.gui_comp_tree;
                    let frame = get_frame(gui_component_tree, root_key);
                    let wheel = info.event.wheel() * 0.1;

                    if frame
                        .horizontal_scroll_area
                        .is_point_inside(frame.last_known_mouse_pos)
                    {
                        frame.percentages[0] += wheel;
                        frame.percentages[0] = frame.percentages[0].clamp(0.0, 1.0);

                        let uv = frame.percentages;

                        scroll_elements(gui_component_tree, root_key, uv);
                    }
                    None
                }),
            )
            .with_listener_advanced(
                GuiEventKind::OnMouseMove,
                Box::new(|info| {
                    let root_key = info.key;
                    let gui_component_tree = info.gui_comp_tree;
                    let mouse_pos = info.event.mouse_pos();
                    let frame = get_frame(gui_component_tree, root_key);
                    frame.last_known_mouse_pos = mouse_pos;
                    None
                }),
            )
        } else {
            self
        }
    }

    pub fn with_visibility(mut self, visibility: bool) -> Self {
        self.state.as_mut().unwrap().is_visible = visibility;
        self
    }
}

impl<'a, ProgramState> HasComponentBuilder<ProgramState> for FrameBuilder<'a, ProgramState> {
    type ComponentKind = FrameState;

    fn manager(&mut self) -> &mut GuiManager<ProgramState> {
        self.manager
    }

    fn parent(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.parent
    }

    fn key(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.frame_key
    }

    fn build(mut self) -> GuiComponentKey {
        let frame_id = self.frame_key.expect("frame key missing");
        let parent_id = self.parent.expect("parent not set");
        let frame_state = self.state.take().expect("frame_state not set");

        self.manager
            .gui_component_tree
            .set_parent(frame_id, parent_id);

        *self.manager.gui_component_tree.get_mut_uninit(frame_id) =
            MaybeUninit::new(Box::new(frame_state));
        self.manager.gui_component_tree.reconstruct_preorder();

        frame_id
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn builder_frame(&mut self) -> FrameBuilder<ProgramState> {
        FrameBuilder::new(self)
    }
}

fn get_frame<'a>(
    tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    key: GuiComponentKey,
) -> &'a mut FrameState {
    tree.get_mut(key)
        .expect("root key invalid")
        .as_any_mut()
        .downcast_mut::<FrameState>()
        .expect("node expected to be a frame")
}

fn translate_children<'a>(
    tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
    disp: Vec2<f32>,
) {
    for NodeInfoMut { val, .. } in tree.iter_children_mut(root_key).skip(1) {
        val.translate(disp);
    }
    let frame = get_frame(tree, root_key);
    frame.camera += disp;
    frame.components_aabb.translate(disp);
}

fn scroll_elements(
    gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
    uv: Vec2<f32>,
) {
    let frame = get_frame(gui_component_tree, root_key);
    let new_min = frame.rails.unwrap().eval(uv.x(), uv.y());
    let disp = new_min - frame.components_aabb.min_pos;
    frame.percentages = uv;
    translate_children(gui_component_tree, root_key, disp)
}

fn compute_component_bounds(
    gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
) -> AABB2<f32> {
    let mut aabb = AABB2::flipped_infinity();
    for NodeInfoMut { val, .. } in gui_component_tree.iter_children_mut(root_key).skip(1) {
        let &pos = val.rel_position();
        let bounds = val.get_bounds();
        let rel_aabb = AABB2::from_point_and_lengths(pos, bounds);
        aabb.merge(rel_aabb);
    }
    aabb
}

fn resize_component_bounds_if_needed(
    gui_component_tree: &mut LinearTree<Box<dyn GuiComponent>>,
    root_key: GuiComponentKey,
) {
    let new_component_bounding_box = compute_component_bounds(gui_component_tree, root_key);
    let old_component_bounding_box = get_frame(gui_component_tree, root_key).components_aabb;
    const EPSILON: f32 = 0.001;

    let component_bounding_box_changed_dramatically = new_component_bounding_box
        .dims()
        .iter()
        .zip(old_component_bounding_box.dims().iter())
        .all(|(&cur, &prev)| (cur - prev).abs() < EPSILON)
        == false;

    if component_bounding_box_changed_dramatically {
        let old_uv = get_frame(gui_component_tree, root_key).percentages;
        scroll_elements(gui_component_tree, root_key, Vec2::zero());

        let frame = get_frame(gui_component_tree, root_key);
        frame.rails = None;
        frame.components_aabb = new_component_bounding_box;
        frame.update_component_bounds_assuming_new_bounds_already_set();

        scroll_elements(gui_component_tree, root_key, old_uv);
    }
}
