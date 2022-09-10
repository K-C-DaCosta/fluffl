use super::*;

pub struct OriginState {
    common: GuiCommonState, 
    rel_position: Vec2<f32>,
}
impl OriginState {
    pub fn new() -> Self {
        Self {
            rel_position: Vec2::zero(),
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

    fn get_bounds(&self) -> Vec2<f32> {
        Vec2::zero()
    }
    
    fn set_bounds(&mut self, _bounds: Vec2<f32>) {

    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.rel_position
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.rel_position = pos;
    }

    fn render_entry<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
        _win_w: f32,
        _win_h: f32,
    ) {
        /* not implemented on purpose*/
    }

    fn render_exit<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
        _win_w: f32,
        _win_h: f32,
    ) {
        /* not implemented on purpose  */
    }
}
