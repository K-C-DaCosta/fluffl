use super::*;

use crate::math::AABB2;
mod button;
mod frame;
mod origin;
mod slider;
pub use self::{button::Button, frame::Frame, origin::Origin};
use std::any::Any;

#[derive(Copy, Clone, Debug)]
pub enum CompSignalKind {
    OnHoverIn = 0isize,
    OnHoverOut = 1,
    OnDrag = 2,
    OnClick = 3,
    OnRelease = 4,
}

#[derive(Copy, Clone)]
pub struct ComponentEventListener {
    pub kind: CompSignalKind,
    pub callback: ListenerCallBack,
}
impl ComponentEventListener {
    pub fn new(kind: CompSignalKind, callback: ListenerCallBack) -> Self {
        Self { kind, callback }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComponentEventSignal {
    pub component_key: GuiComponentKey,
    pub listener_kind: CompSignalKind,
    pub window_event_kind: EventKind,
}
impl ComponentEventSignal {
    pub fn new(sig_kind: CompSignalKind, comp_key: GuiComponentKey, win_event: EventKind) -> Self {
        Self {
            component_key: comp_key,
            listener_kind: sig_kind,
            window_event_kind: win_event,
        }
    }
}

#[derive(Copy, Clone)]
pub struct RenderState<'a> {
    pub global_position: Vec4<f32>,
    pub renderer: &'a GuiRenderer,
    pub gui_component_tree: &'a LinearTree<Box<dyn GuiComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
}

pub struct EventListenerInfo<'a> {
    pub event: EventKind,
    pub key: GuiComponentKey,
    pub gui_comp_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
}

pub trait GuiComponent {
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
}
