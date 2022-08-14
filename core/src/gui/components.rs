use super::*;

pub mod frame;
pub mod button; 

use std::any::Any;

#[derive(Copy, Clone)]
pub enum ComponentEventSignal {
    MouseIn(GuiComponentKey),
    MouseOut(GuiComponentKey),
    MouseMotion(GuiComponentKey, Vec2<FP32>),
}

pub trait GUIComponent {
    fn as_any(&self) -> &dyn Any;
    fn key(&self) -> GuiComponentKey;
    fn window_event(
        &mut self,
        manager: &mut GUIManager,
        event: EventKind,
    );
    fn render(&self, gl: &GlowGL);
}
