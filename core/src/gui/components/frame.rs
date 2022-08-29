use super::*;

#[derive(Clone)]
pub struct FrameState {
    pub bounds: Vec2<f32>,
    pub rel_pos: Vec2<f32>,
    pub color: Vec4<f32>,
    pub edge_color: Vec4<f32>,
    pub roundness: Vec4<f32>,
    pub is_visible: bool,


    
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
        }
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
        r.builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_roundness_vec(self.roundness)
            .set_edge_color(self.edge_color)
            .set_background_color(self.color)
            .set_null_color([0., 0., 0., 0.0])
            .set_bounds(self.bounds)
            .set_position(state.global_position, Vec4::to_pos(self.bounds))
            .render();
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
        let frame_key = Some(manager.add_component_deferred(GuiComponentKey::default(), None));
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

        *self.manager.gui_component_tree.get_mut_opt(frame_id) = Some(Box::new(frame_state));
        self.manager.gui_component_tree.reconstruct_preorder();

        frame_id
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn builder_frame(&mut self) -> FrameBuilder<ProgramState> {
        FrameBuilder::new(self)
    }
}
