use super::*;

pub struct Origin {
    key: GuiComponentKey,
    rel_position: Vec2<f32>,
}
impl Origin {
    pub fn new() -> Self {
        Self {
            key: GuiComponentKey::default(),
            rel_position: Vec2::zero(),
        }
    }
}

impl GUIComponent for Origin {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_bounds(&self) -> Vec2<f32> {
        Vec2::zero()
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.rel_position
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.rel_position = pos;
    }

    fn key(&self) -> GuiComponentKey {
        self.key
    }

    fn set_key(&mut self, key: GuiComponentKey) {
        self.key = key;
    }

    fn render<'a>(&self, gl: &GlowGL, state: RenderState<'a>, win_w: f32, win_h: f32) {}
    
    fn handle_window_event(&mut self, manager: &mut GUIManager,signal:ComponentEventSignal) {
        
    }

    fn set_listener<'a>(&mut self, listener:ComponentEventListener<'a>) {
        
    }
}
