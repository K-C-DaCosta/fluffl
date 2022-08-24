use super::*;
pub struct FrameState {
    pub bounds: Vec2<f32>,
    pub rel_pos: Vec2<f32>,
    pub color: Vec4<f32>,
    pub edge_color: Vec4<f32>,
    pub roundness: Vec4<f32>,
    pub caption: String,
    pub alignment: [TextAlignment; 2],
    pub font_size: f32,
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
            caption: String::new(),
            is_visible: true,
            font_size: 30.0,
            alignment: [TextAlignment::Right, TextAlignment::Center],
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

    pub fn with_caption(mut self, caption: String) -> Self {
        self.caption = caption;
        self
    }

    pub fn with_alignment(mut self, horizontal: TextAlignment, vertical: TextAlignment) -> Self {
        self.alignment = [horizontal, vertical];
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl GuiComponent for FrameState {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
        &self,
        gl: &GlowGL,
        state: RenderState<'b>,
        text_writer: &mut TextWriter,
        win_w: f32,
        win_h: f32,
    ) {
        if self.is_visible == false {
            return;
        }
        let position = state.global_position;
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

        let text = self.caption.as_ref();
        let text_size = self.font_size;
        let aabb = text_writer.calc_text_aabb(text, 0.0, 0.0, text_size);

        let aligned_global_position = compute_alignment_position(
            Vec2::convert(position),
            Vec2::from([aabb.w, aabb.h]),
            self.bounds,
            &self.alignment,
        );

        // if text.is_empty() == false {
        //     text_writer.draw_text_line(
        //         text,
        //         aligned_global_position.x(),
        //         aligned_global_position.y(),
        //         text_size,
        //         Some((win_w as u32, win_h as u32)),
        //     );
        //     unsafe {
        //         //re-enable
        //         gl.enable(glow::BLEND);
        //     }
        // }
    }
}

pub fn compute_alignment_position(
    global_position: Vec2<f32>,
    text_bounds: Vec2<f32>,
    component_bounds: Vec2<f32>,
    alignment: &[TextAlignment; 2],
) -> Vec2<f32> {
    let mut res = Vec2::zero();
    for pos_idx in 0..2 {
        let comp_gpos = global_position[pos_idx];
        let comp_dim = component_bounds[pos_idx];
        let text_dim = text_bounds[pos_idx];
        let alignment_mode = alignment[pos_idx];
        res[pos_idx] = match alignment_mode {
            TextAlignment::Left | TextAlignment::Stretch => comp_gpos,
            TextAlignment::Right => comp_gpos + comp_dim - text_dim,
            TextAlignment::Center => comp_gpos + comp_dim * 0.5 - text_dim * 0.5,
        };
    }
    res
}
