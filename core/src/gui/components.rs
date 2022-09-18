use super::{builder::*, *};

use crate::{extras::text_writer::TextWriter, math::AABB2};
use std::any::Any;

pub mod component_flags;
mod frame;
mod label;
mod origin;
mod slider;
mod textbox;

use self::{component_flags::ComponentFlags, label::LabelState};
pub use self::{frame::*, label::*, origin::*, slider::*, textbox::*};

pub struct TextAligner2D {
    alignment_mode_per_axis: [TextAlignment; 2],
}
impl TextAligner2D {
    pub fn new() -> Self {
        Self {
            alignment_mode_per_axis: [TextAlignment::Center; 2],
        }
    }
    pub fn compute_position(
        &self,
        global_position: Vec2<f32>,
        text_bounds: Vec2<f32>,
        component_bounds: Vec2<f32>,
    ) -> Vec2<f32> {
        let mut res = Vec2::zero();
        for pos_idx in 0..res.len() {
            let comp_gpos = global_position[pos_idx];
            let comp_dim = component_bounds[pos_idx];
            let text_dim = text_bounds[pos_idx];
            let alignment_mode = self.alignment_mode_per_axis[pos_idx];
            res[pos_idx] = match alignment_mode {
                TextAlignment::Left | TextAlignment::Stretch => comp_gpos,
                TextAlignment::Right => comp_gpos + comp_dim - text_dim,
                TextAlignment::Center => comp_gpos + (comp_dim - text_dim) * 0.5,
            };
        }
        res
    }
}

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

impl <ProgramState> ComponentEventListener <ProgramState>{
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
    pub level: i32,
    pub key: GuiComponentKey,
    pub gui_component_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
    pub win_w: f32,
    pub win_h: f32,
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
        level: i32,
        gui_component_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
        key_to_aabb_table: &'a HashMap<GuiComponentKey, math::AABB<2, f32>>,
        window_width: f32,
        window_height: f32,
    ) -> Self {
        Self {
            key,
            global_position,
            gui_component_tree,
            renderer,
            level,
            key_to_aabb_table,
            win_h: window_height,
            win_w: window_width,
        }
    }
}

pub struct EventListenerInfo<'a, ProgramState> {
    pub event: EventKind,
    pub key: GuiComponentKey,
    pub gui_comp_tree: &'a mut LinearTree<Box<dyn GuiComponent>>,
    pub key_to_aabb_table: &'a HashMap<GuiComponentKey, AABB2<f32>>,
    pub mutation_queue: &'a mut MutationRequestQueue<ProgramState>,
}

impl<'a, ProgramState> Into<&'a mut LinearTree<Box<dyn GuiComponent>>>
    for EventListenerInfo<'a, ProgramState>
{
    fn into(self) -> &'a mut LinearTree<Box<dyn GuiComponent>> {
        self.gui_comp_tree
    }
}

#[derive(Clone)]
pub struct GuiCommonState {
    rel_pos: Vec2<f32>,
    bounds: Vec2<f32>,
    flags: ComponentFlags,
    name: String,
}

impl GuiCommonState {
    pub fn new() -> Self {
        Self {
            flags: ComponentFlags::default(),
            name: String::new(),
            rel_pos: Vec2::zero(),
            bounds: Vec2::zero(),
        }
    }

    pub fn with_flags(mut self, flags: ComponentFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_bounds<T: Into<Vec2<f32>>>(mut self, bounds: T) -> Self {
        self.bounds = bounds.into();
        self
    }
}

pub trait GuiComponent {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn common(&self) -> &GuiCommonState;
    fn common_mut(&mut self) -> &mut GuiCommonState;

    fn name(&self) -> &str {
        self.common().name.as_str()
    }
    
    fn set_name(&mut self, name: &str) {
        let common = self.common_mut();
        common.name.clear();
        common.name.push_str(name);
    }

    fn flags(&self) -> &ComponentFlags {
        &self.common().flags
    }

    fn flags_mut(&mut self) -> &mut ComponentFlags {
        &mut self.common_mut().flags
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.common().rel_pos
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.common_mut().rel_pos = pos;
    }

    fn bounds(&self) -> Vec2<f32> {
        self.common().bounds
    }

    fn set_bounds(&mut self, bounds: Vec2<f32>) {
        self.common_mut().bounds = bounds;
    }

    fn is_visible(&self) -> bool {
        self.flags().is_set(component_flags::VISIBLE)
    }

    fn set_visible(&mut self, is_visible: bool) {
        self.flags_mut().unset(component_flags::VISIBLE);
        self.flags_mut()
            .set(component_flags::VISIBLE & ComponentFlags::as_mask(is_visible))
    }

    fn set_overflowable(&mut self, overflowable: bool) {
        self.flags_mut().unset(component_flags::OVERFLOWABLE);
        self.flags_mut()
            .set(component_flags::OVERFLOWABLE & ComponentFlags::as_mask(overflowable))
    }

    fn is_overflowable(&self) -> bool {
        self.flags().is_set(component_flags::OVERFLOWABLE)
    }

    fn get_aabb(&self, global_x0: Vec4<f32>) -> AABB2<f32> {
        let bounds = self.bounds();
        AABB2::from_point_and_lengths(Vec2::convert(global_x0), bounds)
    }

    fn translate(&mut self, disp: Vec2<f32>) {
        let &pos = self.rel_position();
        self.set_rel_position(pos + disp);
    }

    fn is_origin(&self) -> bool {
        self.as_any().downcast_ref::<OriginState>().is_some()
    }

    /// this fires the first occurrence in the tree
    fn render_entry<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
    );

    /// this fires after everything in the component subtree has been rendered
    fn render_exit<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
    );
}

const LAYER_BIAS: i32 = 128;
/// used
pub fn layer_lock(gl: &GlowGL, layer_id: i32, flags: ComponentFlags) {
    if flags.is_set(component_flags::OVERFLOWABLE) == false {
        layer_lock_always(gl, layer_id);
    } else {
        unsafe {
            gl.enable(glow::STENCIL_TEST);
            gl.stencil_mask(0xff);
            gl.stencil_func(glow::ALWAYS, (layer_id + 0) + LAYER_BIAS, 0xff);
            gl.stencil_op(glow::REPLACE, glow::REPLACE, glow::REPLACE);
        }
    }
}

pub fn layer_lock_always(gl: &GlowGL, layer_id: i32) {
    if layer_id == 1 {
        //initalize the stencil buffer for the first layer
        unsafe {
            gl.enable(glow::STENCIL_TEST);
            gl.stencil_mask(0xff);
            gl.stencil_func(glow::ALWAYS, (layer_id + 0) + LAYER_BIAS, 0xff);
            gl.stencil_op(glow::REPLACE, glow::REPLACE, glow::REPLACE);
        }
    } else {
        // layer_id -1 is the parent of the current layer.
        // the goal i  to clip away pixels OUTSIDE of the parents domain
        unsafe {
            gl.enable(glow::STENCIL_TEST);
            gl.stencil_mask(0xff);
            gl.stencil_func(glow::LEQUAL, (layer_id - 1) + LAYER_BIAS, 0xff);
            gl.stencil_op(glow::KEEP, glow::INCR, glow::INCR);
        }
    }
}

pub fn layer_unlock(gl: &GlowGL) {
    unsafe {
        gl.disable(glow::STENCIL_TEST);
    }
}
