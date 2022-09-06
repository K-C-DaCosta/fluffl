use super::{builder::*, *};

use crate::{extras::text_writer::TextWriter, math::AABB2};
use std::any::Any;

mod frame;
mod origin;
mod slider;
mod textbox;

pub use self::{frame::*, origin::*, slider::*, textbox::*};

#[derive(Copy, Clone)]
#[rustfmt::skip]
pub enum TextAlignment {
    Left    = 0,
    Center  = 1,
    Right   = 2,
    Stretch = 3, 
}

#[derive(Copy, Clone, Debug)]
#[rustfmt::skip]
pub enum GuiEventKind {
    OnHoverIn           =  0,
    OnHoverOut          =  1,
    OnDrag              =  2,
    OnMouseDown         =  3,
    OnMouseRelease      =  4,
    OnMouseMove         =  5,
    OnKeyDown           =  6,
    OnKeyRelease        =  7,
    OnFocusIn           =  8, 
    OnFocusOut          =  9,
    OnWheelWhileFocused = 10, 
    OnWheelWhileHovered = 11,
}

pub struct ComponentEventListener<ProgramState> {
    pub kind: GuiEventKind,
    pub callback: ListenerCallBack<ProgramState>,
}

impl<ProgramState> ComponentEventListener<ProgramState> {
    pub const fn new(kind: GuiEventKind, callback: ListenerCallBack<ProgramState>) -> Self {
        Self { kind, callback }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComponentEventSignal {
    pub component_key: GuiComponentKey,
    pub listener_kind: GuiEventKind,
    pub window_event_kind: EventKind,
}
impl ComponentEventSignal {
    pub fn new(sig_kind: GuiEventKind, comp_key: GuiComponentKey, win_event: EventKind) -> Self {
        Self {
            component_key: comp_key,
            listener_kind: sig_kind,
            window_event_kind: win_event,
        }
    }
}

pub struct RenderState<'a> {
    pub global_position: Vec4<f32>,
    pub renderer: &'a GuiRenderer,
    pub level: usize,
    pub key: GuiComponentKey,
    pub gui_component_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
}

impl<'a> Clone for RenderState<'a> {
    fn clone(&self) -> Self {
        //literally just shallow copies the struct 
        unsafe { std::mem::transmute_copy(self) }
    }
}

impl<'a> RenderState<'a> {
    pub fn new(
        key: GuiComponentKey,
        global_position: Vec4<f32>,
        renderer: &'a GuiRenderer,
        level: usize,
        gui_component_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, math::AABB<2, f32>>,
    ) -> Self {
        Self {
            key,
            global_position,
            gui_component_tree,
            renderer,
            level,
            key_to_aabb_table,
        }
    }
}

pub struct EventListenerInfo<'a, ProgramState> {
    pub state: &'a ProgramState,
    pub event: EventKind,
    pub key: GuiComponentKey,
    pub gui_comp_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
}

impl<'a, ProgramState> Into<&'a mut LinearTree<Box<dyn GuiComponent>>>
    for EventListenerInfo<'a, ProgramState>
{
    fn into(self) -> &'a mut LinearTree<Box<dyn GuiComponent>> {
        self.gui_comp_tree
    }
}

pub trait GuiComponent {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, is_visible: bool);

    fn get_bounds(&self) -> Vec2<f32>;

    fn set_bounds(&mut self, bounds: Vec2<f32>);

    fn rel_position(&self) -> &Vec2<f32>;

    fn set_rel_position(&mut self, pos: Vec2<f32>);

    fn render<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
        win_w: f32,
        win_h: f32,
    );

    fn get_aabb(&self, global_x0: Vec4<f32>) -> AABB2<f32> {
        let bounds = self.get_bounds();
        AABB2::from_point_and_lengths(Vec2::convert(global_x0), bounds)
    }

    fn translate(&mut self, disp: Vec2<f32>) {
        let &pos = self.rel_position();
        self.set_rel_position(pos + disp);
    }

    fn is_origin(&self) -> bool {
        self.as_any().downcast_ref::<OriginState>().is_some()
    }
}

/// used
pub fn layer_lock(gl: &GlowGL, layer_id: usize) {
    unsafe {
        gl.enable(glow::STENCIL_TEST);
        gl.stencil_mask(0xff);
        gl.stencil_func(glow::LEQUAL, (layer_id as i32) - 1, 0xff);
        gl.stencil_op(glow::KEEP, glow::INCR, glow::INCR);
    }
}

pub fn layer_unlock(gl: &GlowGL) {
    unsafe {
        gl.disable(glow::STENCIL_TEST);
    }
}
