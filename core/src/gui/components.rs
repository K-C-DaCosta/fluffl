use super::*;
use crate::math::AABB2;
mod button;
mod frame;
mod origin;
mod slider;

use std::any::Any;

pub use self::{button::Button, frame::Frame, origin::Origin};

#[derive(Copy, Clone)]
pub enum ComponentEventListener<'a> {
    OnHoverIn(fn(MouseEventInfo<'a>)),
    OnHoverOut(fn(MouseEventInfo<'a>)),
    OnDrag(fn(MouseEventInfo<'a>)),
    OnClick(fn(MouseEventInfo<'a>)),
    OnRelease(fn(MouseEventInfo<'a>)),
}

#[derive(Copy, Clone,Debug)]
pub enum ComponentEventSignal {
    HoverIn(GuiComponentKey, EventKind),
    HoverOut(GuiComponentKey, EventKind),
    Drag(GuiComponentKey, EventKind),
    OnClick(GuiComponentKey, EventKind),
    OnRelease(GuiComponentKey, EventKind),
}

#[derive(Copy, Clone)]
pub struct RenderState<'a> {
    pub global_position: Vec4<f32>,
    pub renderer: &'a GuiRenderer,
    pub gui_component_tree: &'a LinearTree<Box<dyn GUIComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
}

pub struct MouseEventInfo<'a> {
    pub key: GuiComponentKey,
    pub manager: &'a mut GUIManager,
    pub mouse_pos: Vec2<f32>,
    pub mouse_disp: Vec2<f32>,
}

pub trait GUIComponent {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_bounds(&self) -> Vec2<f32>;
    fn rel_position(&self) -> &Vec2<f32>;
    fn set_rel_position(&mut self, pos: Vec2<f32>);
    fn key(&self) -> GuiComponentKey;
    fn set_key(&mut self, key: GuiComponentKey);
    fn handle_window_event(&mut self, manager: &mut GUIManager, signal: ComponentEventSignal);

    fn render<'a>(&self, gl: &GlowGL, state: RenderState<'a>, win_w: f32, win_h: f32);

    fn get_aabb(&self, global_x0: Vec4<f32>) -> AABB2<f32> {
        let bounds = self.get_bounds();
        AABB2::from_point_and_lengths(Vec2::convert(global_x0), bounds)
    }

    fn translate(&mut self, disp: Vec2<f32>) {
        let &pos = self.rel_position();
        self.set_rel_position(pos + disp);
    }

    fn set_listener<'a>(&mut self, listener: ComponentEventListener<'a>);
}
