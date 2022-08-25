use super::*;

pub struct ButtonState {
    width: f32,
    height: f32,
    rel_position: Vec2<f32>,
    color: Vec4<f32>,
    _is_visible: bool,
}

impl GuiComponent for ButtonState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_bounds(&self) -> Vec2<f32> {
        Vec2::from([self.width, self.height])
    }

    fn set_bounds(&mut self, _bounds: Vec2<f32>) {
        unimplemented!("not implemented")
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.rel_position
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.rel_position = pos;
    }

    fn render<'a>(
        &self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
        win_w: f32,
        win_h: f32,
    ) {
        let bounds = self.get_bounds();
        let position = state.global_position;
        let r = state.renderer;

        r.builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_background_color(self.color)
            .set_position(position, Vec4::convert(bounds))
            .set_bounds([self.width, self.height])
            .render();

        text_writer.draw_text_line(
            "HELLO",
            position.x(),
            position.y(),
            12.0,
            Some((win_w as u32, win_h as u32)),
        );
    }
}
