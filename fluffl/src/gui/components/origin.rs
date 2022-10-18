use super::*;


#[derive(Default)]
pub struct OriginState {
    common: GuiCommonState, 
}

impl OriginState {
    pub fn new() -> Self {
        Self {
            common: GuiCommonState::new().with_flags(component_flags::VISIBLE),
        }
    }
}

impl GuiComponent for OriginState {
    
    fn common(&self) -> &GuiCommonState {
        &self.common
    }
    
    fn common_mut(&mut self) -> &mut GuiCommonState {
        &mut self.common
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn render_entry<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
    ) {
        /* not implemented on purpose*/
    }

    fn render_exit<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
    ) {
        /* not implemented on purpose  */
    }
}
