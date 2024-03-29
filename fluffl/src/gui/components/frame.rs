use super::*;
use crate::{collections::flat_nary_tree::NodeInfoMut, window::event_util::MouseCode};

mod scrollbar;

pub const HORIZONTAL_SCROLL_HEIGHT: f32 = 20.0;
pub const VERTICAL_SCROLL_WIDTH: f32 = 20.0;

/// at this point this is basically a 3x3 basis that can be written like:
/// ```text
///       [ horizontal.x , vertical.x , min.x ]  
/// M_3x3=[ horizontal.y , vertical.y , min.y ]
///       [     0        ,    0       ,   1   ]
/// ```
#[derive(Copy, Clone)]
pub struct SliderRail {
    min: Vec2<f32>,
    horizontal_disp: Vec2<f32>,
    vertical_disp: Vec2<f32>,
}
impl SliderRail {
    pub fn eval(&self, u: f32, v: f32) -> Vec2<f32> {
        self.min + self.horizontal_disp * u + self.vertical_disp * v
    }

    pub fn can_draw_horizontal(&self) -> bool {
        self.horizontal_disp.length_squared() > 0.01
    }

    pub fn can_draw_vertical(&self) -> bool {
        self.vertical_disp.length_squared() > 0.01
    }
}
impl Default for SliderRail {
    fn default() -> Self {
        Self {
            min: Vec2::zero(),
            horizontal_disp: Vec2::zero(),
            vertical_disp: Vec2::zero(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum FocusedScrollBarKind {
    Horizontal,
    Vertical,
    Nothing,
}

#[derive(Clone)]
pub struct FrameState {
    pub common: GuiCommonState,
    pub color: Vec4<f32>,
    pub edge_color: Vec4<f32>,
    pub edge_thickness: f32,
    pub roundness: Vec4<f32>,

    is_scrollbars_enabled: bool,
    camera: Vec2<f32>,
    rails: Option<SliderRail>,
    percentages: Vec2<f32>,
    components_aabb: AABB2<f32>,
    horizontal_scroll_area: AABB2<f32>,
    vertical_scroll_area: AABB2<f32>,
    last_known_mouse_pos: Vec2<f32>,
    focused_scrollbar: FocusedScrollBarKind,
}

impl Default for FrameState {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameState {
    pub fn new() -> Self {
        Self {
            common: GuiCommonState::new()
                .with_flags(component_flags::VISIBLE)
                .with_bounds([128.; 2]),
            color: Vec4::rgb_u32(0xF94892),
            edge_color: Vec4::rgb_u32(0x89CFFD),
            edge_thickness: 0.02,
            roundness: Vec4::from([1.0, 1.0, 1.0, 1.0]),
            is_scrollbars_enabled: false,
            camera: Vec2::zero(),
            components_aabb: AABB2::zero(),
            horizontal_scroll_area: AABB2::zero(),
            vertical_scroll_area: AABB2::zero(),
            rails: Some(SliderRail::default()),
            percentages: Vec2::zero(),
            last_known_mouse_pos: Vec2::zero(),
            focused_scrollbar: FocusedScrollBarKind::Nothing,
        }
    }

    fn draw_rectangle<T>(
        gl: &GlowGL,
        r: &GuiRenderer,
        win_w: f32,
        win_h: f32,
        rect: AABB2<f32>,
        roundness: T,
        depth: f32,
    ) where
        T: Into<Vec4<f32>>,
    {
        // unsafe {
        //     gl.blend_func(glow::ONE, glow::ONE);
        // }
        let mut position = Vec4::convert(rect.min_pos);
        position[2] = depth;

        r.builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_roundness_vec(roundness.into())
            .set_edge_color(Vec4::rgb_u32(!0))
            .set_edge_thickness(2.0)
            .set_background_color(Vec4::rgb_u32(0))
            .set_bounds(rect.dims())
            .set_position(position, Vec4::convert(rect.dims()))
            .render();

        unsafe {
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
    }

    fn update_component_bounds_assuming_new_bounds_already_set(&mut self) {
        if self.rails.is_some() {
            return;
        }
        // overflow vectors for the "right" and "down" half-spaces of the frame
        // we don't care about overflow on the "left" and "top" half-spaces
        let h_overlap = self.bounds().x() - self.components_aabb.max_pos.x();
        let v_overlap = self.bounds().y() - self.components_aabb.max_pos.y();

        let h_mask = (v_overlap.min(0.0).abs() > 0.01) as u32 as f32;
        let v_mask = (h_overlap.min(0.0).abs() > 0.01) as u32 as f32;

        let horizontal_disp =
            Vec2::from([(h_overlap - VERTICAL_SCROLL_WIDTH * h_mask).min(0.0), 0.0]);

        let vertical_disp = Vec2::from([
            0.0,
            (v_overlap - HORIZONTAL_SCROLL_HEIGHT * v_mask).min(0.0),
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

    fn common(&self) -> &GuiCommonState {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GuiCommonState {
        &mut self.common
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn render_entry(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'_>,
        _text_writer: &mut TextWriter,
    ) {
        if !self.is_visible() {
            return;
        }

        let r = state.renderer;
        let level = state.level;
        let pos = Vec2::convert(state.global_position);
        let win_w = state.win_w;
        let win_h = state.win_h;

        layer_lock(gl, level, *self.flags());

        r.builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_roundness_vec(self.roundness)
            .set_edge_color(self.edge_color)
            .set_background_color(self.color)
            .set_edge_thickness(self.edge_thickness)
            .set_bounds(self.bounds())
            .set_position(state.global_position, Vec4::to_pos(self.bounds()))
            .render();

        layer_unlock(gl);

        //compute global horizontal bounding box
        let can_draw_horizontal = self
            .rails
            .map(|r| r.can_draw_horizontal())
            .unwrap_or_default();
        let can_draw_vertical = self
            .rails
            .map(|r| r.can_draw_vertical())
            .unwrap_or_default();

        self.horizontal_scroll_area = AABB2::from_point_and_lengths(
            Vec2::from([
                pos.x(),
                pos.y() + self.bounds().y() - HORIZONTAL_SCROLL_HEIGHT,
            ]),
            Vec2::from([self.bounds().x(), HORIZONTAL_SCROLL_HEIGHT]),
        );

        if can_draw_vertical && !can_draw_horizontal {
            //take the full height
            self.vertical_scroll_area = AABB2::from_point_and_lengths(
                Vec2::from([pos.x() + self.bounds().x() - VERTICAL_SCROLL_WIDTH, pos.y()]),
                Vec2::from([VERTICAL_SCROLL_WIDTH, self.bounds().y()]),
            );
        } else {
            self.vertical_scroll_area = AABB2::from_point_and_lengths(
                Vec2::from([pos.x() + self.bounds().x() - VERTICAL_SCROLL_WIDTH, pos.y()]),
                Vec2::from([
                    VERTICAL_SCROLL_WIDTH,
                    self.bounds().y() - HORIZONTAL_SCROLL_HEIGHT,
                ]),
            );
        }
    }

    fn render_exit(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'_>,
        _text_writer: &mut TextWriter,
    ) {
        let r = state.renderer;
        let win_w = state.win_w;
        let win_h = state.win_h;

        let can_draw_horizontal = self
            .rails
            .map(|r| r.can_draw_horizontal())
            .unwrap_or_default();

        let can_draw_vertical = self
            .rails
            .map(|r| r.can_draw_vertical())
            .unwrap_or_default();

        let can_draw_hori_mask = can_draw_horizontal as u32 as f32;
        let _can_draw_vert_mask = can_draw_vertical as u32 as f32;

        if self.is_scrollbars_enabled {
            const SCROLLBAR_DEPTH: f32 = 0.0;
            //draw horizontal scroll bar
            if can_draw_horizontal {
                Self::draw_rectangle(
                    gl,
                    r,
                    win_w,
                    win_h,
                    self.horizontal_scroll_area,
                    self.roundness,
                    SCROLLBAR_DEPTH,
                );
            }

            let s0 = self.horizontal_scroll_area.min_pos;
            let u = self.percentages.x();
            let cursor_bounds_h = Vec2::from([15.0, HORIZONTAL_SCROLL_HEIGHT]);
            let cursor_pos = Vec2::from([
                (self.bounds().x() - cursor_bounds_h.x()) * u + s0.x(),
                s0.y(),
            ]);

            let cursor_aabb = AABB2::from_point_and_lengths(cursor_pos, cursor_bounds_h);

            if can_draw_horizontal {
                Self::draw_rectangle(
                    gl,
                    r,
                    win_w,
                    win_h,
                    cursor_aabb,
                    self.roundness,
                    SCROLLBAR_DEPTH,
                );
            }

            //draw vertical scroll bar
            if can_draw_vertical {
                Self::draw_rectangle(
                    gl,
                    r,
                    win_w,
                    win_h,
                    self.vertical_scroll_area,
                    [0.0; 4],
                    SCROLLBAR_DEPTH,
                );
            }

            let s0 = self.vertical_scroll_area.min_pos;
            let v = self.percentages.y();
            let cursor_bounds_v = Vec2::from([VERTICAL_SCROLL_WIDTH, HORIZONTAL_SCROLL_HEIGHT]);
            let cursor_pos = Vec2::from([
                s0.x(),
                (self.bounds().y()
                    - cursor_bounds_v.y()
                    - (cursor_bounds_h.y() * can_draw_hori_mask))
                    * v
                    + s0.y(),
            ]);

            let cursor_aabb = AABB2::from_point_and_lengths(cursor_pos, cursor_bounds_v);

            if can_draw_vertical {
                Self::draw_rectangle(gl, r, win_w, win_h, cursor_aabb, [1.0; 4], SCROLLBAR_DEPTH);
            }
        }
    }
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
        self.state.as_mut().unwrap().set_bounds(bounds);
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

    pub fn with_edge_thickness<T: Into<f32>>(mut self, edge_thickness: T) -> Self {
        self.state.as_mut().unwrap().edge_thickness = edge_thickness.into();
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
        self.state
            .as_mut()
            .unwrap()
            .set_rel_position(Vec2::from(pos));
        self
    }

    pub fn with_flags(mut self, flags: ComponentFlags) -> Self {
        self.state.as_mut().unwrap().flags_mut().set(flags);
        self
    }

    pub fn with_scrollbars(mut self, enable: bool) -> Self {
        if enable {
            self.state.as_mut().unwrap().is_scrollbars_enabled = true;
            self.with_listener_advanced(GuiEventKind::OnMouseDown, scrollbar::mousedown())
                .with_listener_advanced(GuiEventKind::OnMouseRelease, scrollbar::mouseup())
                .with_listener_advanced(GuiEventKind::OnDrag, scrollbar::drag())
                .with_listener_advanced(GuiEventKind::OnWheelWhileHovered, scrollbar::wheel())
                .with_listener_advanced(GuiEventKind::OnMouseMove, scrollbar::mousemove())
        } else {
            self
        }
    }

    pub fn with_visibility(mut self, visibility: bool) -> Self {
        self.state.as_mut().unwrap().set_visible(visibility);
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

    fn state(&mut self) -> &mut Option<Self::ComponentKind> {
        &mut self.state
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

        // to make sure the scrollbars get updated on the first draw
        self.manager
            .component_signal_bus
            .push_back(ComponentEventSignal::new(
                GuiEventKind::OnWheelWhileHovered,
                frame_id,
                EventKind::MouseWheel {
                    button_code: MouseCode::WHEEL { direction: 0 },
                },
            ));

        frame_id
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn builder_frame(&mut self) -> FrameBuilder<ProgramState> {
        FrameBuilder::new(self)
    }
}
