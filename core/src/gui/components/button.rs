use super::*;

pub struct Button {
    key: GuiComponentKey,
    width: f32,
    height: f32,
    rel_position: Vec2<f32>,
    color: Vec4<f32>,
    is_visible: bool,
}

impl GUIComponent for Button {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self    
    }


    fn get_bounds(&self)-> Vec2<f32> {
        Vec2::from([self.width,self.height])
    }

    fn rel_position(&self)->&Vec2<f32> {
        &self.rel_position
    }

    fn key(&self) -> GuiComponentKey {
        self.key
    }

    fn set_key(&mut self,key:GuiComponentKey) {
        self.key = key;
    }
    
    fn set_rel_position(&mut self,pos:Vec2<f32>) {
        self.rel_position = pos;     
    }
    
    fn handle_window_event(&mut self, manager: &mut GUIManager,signal:ComponentEventSignal) {
        
    }

    fn render<'a>(&self, gl: &GlowGL,state:RenderState<'a>, win_w: f32, win_h: f32) {
        
    }

    fn set_listener<'a>(&mut self, listener:ComponentEventListener<'a>) {
        
    }
}
