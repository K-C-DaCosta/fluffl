use super::*;
use crate::math::AABB2;
mod button;
mod frame;
mod origin;
mod slider;

use std::any::Any;

pub use self::{button::Button, frame::Frame, origin::Origin};

#[derive(Copy, Clone)]
pub enum ComponentEventSignal {
    MouseIn(GuiComponentKey),
    MouseOut(GuiComponentKey),
    MouseMotion(GuiComponentKey, Vec2<f32>),
}

pub trait GUIComponent {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_bounds(&self) -> Vec2<f32>;
    fn rel_position(&self) -> &Vec2<f32>;
    fn set_rel_position(&mut self, pos: Vec2<f32>);
    fn key(&self) -> GuiComponentKey;
    fn set_key(&mut self, key: GuiComponentKey);
    
    fn window_event(&mut self, manager: &mut GUIManager, event: EventKind);
    
    fn render(&self, gl: &GlowGL, r: &GuiRenderer, s: &MatStack<f32>, win_w: f32, win_h: f32);

    fn get_aabb(&self, global_x0: Vec4<f32>) -> AABB2<f32> {
        let bounds = self.get_bounds();
        AABB2::from_point_and_lengths(Vec2::convert(global_x0), bounds)
    }
    
    fn translate(&mut self, disp: Vec2<f32>) {
        let &pos = self.rel_position();
        self.set_rel_position(pos + disp);
    }
}
