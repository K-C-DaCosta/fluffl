use crate::extras::math_util::AABB;

use super::*;

struct CaptionClipper {
    prev_cap_len: usize,
    visible_text: String,
}

impl CaptionClipper {
    fn clip_text<'a>(
        &'a mut self,
        text: &'a str,
        font_size: f32,
        frame_bounds: Vec2<f32>,
        text_writer: &TextWriter,
    ) -> &'a str {
        if text.is_empty() == true {
            return "";
        }

        //use cached results if caption hasn't changed
        if text.len() == self.prev_cap_len {
            return &self.visible_text;
        }

        self.clear();

        let mut clipped_text;
        let text_size = font_size;
        let mut lbound = 0;
        let mut range_len = 0;
        let mut aabb;
        let mut char_iter = text.chars();
        let max_text_width = (frame_bounds.x() - 2.0).max(0.0);

        while let Some(_) = char_iter.next() {
            clipped_text = &text[lbound..lbound + range_len];
            aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, text_size);
            if aabb.w < max_text_width {
                range_len += 1;
            } else {
                lbound += 1;
            }
        }
        loop {
            clipped_text = &text[lbound..lbound + range_len];
            aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, text_size);
            if aabb.w >= max_text_width {
                lbound += 1;
                range_len -= 1;
            } else {
                break;
            }
        }

        clipped_text = &text[lbound..lbound + range_len];
        self.visible_text.clear();
        self.visible_text.push_str(clipped_text);
        self.prev_cap_len = text.len();

        clipped_text
    }

    fn clear(&mut self) {
        self.prev_cap_len = 0;
        self.visible_text.clear();
    }
}
pub struct TextBoxState {
    pub frame: FrameState,
    pub alignment: [TextAlignment; 2],
    pub caption: String,
    pub font_size: f32,
    clipper: CaptionClipper,
}
impl TextBoxState {
    pub fn new() -> Self {
        Self {
            frame: FrameState::new(),
            alignment: [TextAlignment::Center; 2],
            caption: String::new(),
            font_size: 12.0,
            clipper: CaptionClipper {
                prev_cap_len: 0,
                visible_text: String::new(),
            },
        }
    }
}

impl GuiComponent for TextBoxState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn is_visible(&self) -> bool {
        self.frame.is_visible
    }
    fn set_visible(&mut self, is_visible: bool) {
        self.frame.is_visible = is_visible;
    }

    fn get_bounds(&self) -> Vec2<f32> {
        self.frame.bounds
    }

    fn set_bounds(&mut self, bounds: Vec2<f32>) {
        self.frame.set_bounds(bounds);
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.frame.rel_pos
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.frame.rel_pos = pos;
    }

    fn render<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
        win_w: f32,
        win_h: f32,
    ) {
        self.frame.render(gl, state, text_writer, win_w, win_h);

        let clipper = &mut self.clipper;
        let caption = &self.caption;
        let frame_bounds = self.frame.bounds;
        let text_size = self.font_size;

        let clipped_text = clipper.clip_text(caption, self.font_size, frame_bounds, text_writer);
        let position = state.global_position;

        if clipped_text.is_empty() == false {
            let aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, text_size);

            let aligned_global_position = compute_alignment_position(
                Vec2::convert(position),
                Vec2::from([aabb.w, aabb.h]),
                self.frame.bounds,
                &self.alignment,
            );

            text_writer.draw_text_line(
                clipped_text,
                aligned_global_position.x()+2.0,
                aligned_global_position.y(),
                text_size,
                Some((win_w as u32, win_h as u32)),
            );

            unsafe {
                //re-enable
                gl.enable(glow::BLEND);
            }
        }
    }
}

pub struct TextBoxBuilder<'a, ProgramState> {
    manager: &'a mut GuiManager<ProgramState>,
    state: Option<TextBoxState>,
    parent_key: Option<GuiComponentKey>,
    textbox_key: Option<GuiComponentKey>,
}
impl<'a, ProgramState> TextBoxBuilder<'a, ProgramState> {
    pub fn new(manager: &'a mut GuiManager<ProgramState>) -> Self {
        let textbox_key = manager.add_component_deferred(GuiComponentKey::default(), None);
        Self {
            manager,
            state: Some(TextBoxState::new()),
            textbox_key: Some(textbox_key),
            parent_key: None,
        }
    }

    pub fn with_bounds<T>(mut self, bounds: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        let bounds = Vec2::from(bounds);
        self.state.as_mut().unwrap().frame.bounds = bounds;
        self
    }

    pub fn with_color<T>(mut self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.state.as_mut().unwrap().frame.color = Vec4::from(color);
        self
    }

    pub fn with_edge_color<T>(mut self, color: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.state.as_mut().unwrap().frame.edge_color = Vec4::from(color);
        self
    }

    pub fn with_roundness<T>(mut self, r: T) -> Self
    where
        Vec4<f32>: From<T>,
    {
        self.state.as_mut().unwrap().frame.roundness = Vec4::from(r);
        self
    }

    pub fn with_position<T>(mut self, pos: T) -> Self
    where
        Vec2<f32>: From<T>,
    {
        self.state.as_mut().unwrap().frame.rel_pos = Vec2::from(pos);
        self
    }

    pub fn with_visibility(mut self, visibility: bool) -> Self {
        self.state.as_mut().unwrap().frame.is_visible = visibility;
        self
    }

    pub fn with_alignment(mut self, alignment: [TextAlignment; 2]) -> Self {
        self.state.as_mut().unwrap().alignment = alignment;
        self
    }

    pub fn with_caption(mut self, caption: String) -> Self {
        self.state.as_mut().unwrap().caption = caption;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.state.as_mut().unwrap().font_size = size;
        self
    }
}

impl<'a, ProgramState> HasComponentBuilder<ProgramState> for TextBoxBuilder<'a, ProgramState> {
    type ComponentKind = TextBoxState;
    fn manager(&mut self) -> &mut GuiManager<ProgramState> {
        self.manager
    }

    fn parent(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.parent_key
    }

    fn key(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.textbox_key
    }

    fn build(self) -> GuiComponentKey {
        let manager = self.manager;
        let textbox_parent_node_id = self.parent_key.unwrap_or_default();
        let textbox_node_id = self.textbox_key.expect("textbox key missing");
        let textbox_state = self.state.expect("textbox state missing");

        // set node state
        *manager.gui_component_tree.get_mut_opt(textbox_node_id) = Some(Box::new(textbox_state));

        // set node parent
        manager
            .gui_component_tree
            .set_parent(textbox_node_id, textbox_parent_node_id);

        // reconstruct parent
        manager.gui_component_tree.reconstruct_preorder();

        textbox_node_id
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn builder_textbox(&mut self) -> TextBoxBuilder<ProgramState> {
        TextBoxBuilder::new(self)
    }
}
