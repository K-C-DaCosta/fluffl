use super::*;

pub struct Frame {
    key: GuiComponentKey,
    width: f32,
    height: f32,
    rel_position: Vec2<FP32>,
    color: Vec4<f32>,
    is_visible: bool,
}

impl GUIComponent for Frame {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn key(&self) -> GuiComponentKey {
        self.key
    }

    fn window_event(&mut self, manager: &mut GUIManager, event: EventKind) {
        
    }

    fn render(&self, gl: &GlowGL) {}
}
