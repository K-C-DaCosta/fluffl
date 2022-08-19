use super::*;

mod button;
mod frame;
mod origin;


use std::any::Any;

pub use self::{
    button::Button, 
    frame::Frame,
    origin::Origin, 
};

#[derive(Copy, Clone)]
pub enum ComponentEventSignal {
    MouseIn(GuiComponentKey),
    MouseOut(GuiComponentKey),
    MouseMotion(GuiComponentKey, Vec2<FP32>),
}

pub trait GUIComponent {
    fn as_any(&self) -> &dyn Any;
    fn rel_position(&self)->&Vec2<f32>;
    fn set_rel_position(&mut self,pos:Vec2<f32>);
    fn key(&self) -> GuiComponentKey;
    fn set_key(&mut self,key:GuiComponentKey);
    fn window_event(&mut self, manager: &mut GUIManager, event: EventKind);
    fn render(&self, gl: &GlowGL, r: &GuiRenderer, s: &MatStack<f32>, win_w: f32, win_h: f32);
}
