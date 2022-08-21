use super::*;

pub struct Frame {
    key: GuiComponentKey,
    pub bounds: Vec2<f32>,
    pub rel_pos: Vec2<f32>,
    pub color: Vec4<f32>,
    pub edge_color: Vec4<f32>,
    pub roundness: Vec4<f32>,
    pub is_visible: bool,
}
impl Frame {
    pub fn new() -> Self {
        Self {
            key: GuiComponentKey::default(),
            bounds: Vec2::from([128.; 2]),
            rel_pos: Vec2::from([0.0; 2]),
            color: Vec4::rgb_u32(0xF94892),
            edge_color: Vec4::rgb_u32(0x89CFFD),
            roundness: Vec4::from([1.0, 1.0, 1.0, 1.0]),
            is_visible: true,
        }
    }

    pub fn with_bounds<T>(mut self, bounds: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        let bounds = Vec2::from(bounds);
        self.bounds = bounds;
        self
    }

    pub fn with_color<T>(mut self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.color = Vec4::from(color);
        self
    }

    pub fn with_edge_color<T>(mut self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.edge_color = Vec4::from(color);
        self
    }

    pub fn with_roundness<T>(mut self, r: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.roundness = Vec4::from(r);
        self
    }

    pub fn with_position<T>(mut self, pos: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        self.rel_pos = Vec2::from(pos);
        self
    }
}

impl GuiComponent for Frame {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_bounds(&self) -> Vec2<f32> {
        self.bounds
    }

    fn key(&self) -> GuiComponentKey {
        self.key
    }

    fn set_key(&mut self, key: GuiComponentKey) {
        self.key = key;
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.rel_pos = pos;
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.rel_pos
    }

    fn render<'b>(&self, gl: &GlowGL, state: RenderState<'b>, win_w: f32, win_h: f32) {
        if self.is_visible == false {
            return;
        }

        let r = state.renderer;
        r.builder(gl, RendererShaderKind::Frame)
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
