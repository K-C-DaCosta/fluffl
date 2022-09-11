use super::*;

pub struct LabelState {
    common: GuiCommonState,
    caption: String,
}
impl LabelState {
    pub fn new() -> Self {
        Self {
            common: GuiCommonState::new().with_flags(component_flags::VISIBLE),
            caption: String::new(),
        }
    }
}

impl GuiComponent for LabelState {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn common(&self) -> &GuiCommonState {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GuiCommonState {
        &mut self.common
    }

    fn render_entry<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
    ) {
        let win_w = state.win_w;
        let win_h = state.win_h;
        layer_lock(gl, state.level, *self.flags());




        layer_unlock(gl);
    }

    fn render_exit<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
    ) {
        /* not implemented on purpose */
    }
}
