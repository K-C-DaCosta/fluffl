use super::*;

use crate::{slice::IdxSlice, time::Instant};

/// Given a string of text, this code figures out what substring can fit inside of rectangle
pub struct CaptionClipper {
    prev_cap_len: usize,
    visible_text: String,
    visible_slice_first_overflow: Option<IdxSlice>,
    visible_slice: IdxSlice,
    visible_text_dx: Vec<f32>,
    scroll_cursor: isize,
    can_off_cursor: Option<isize>,
    scroll_cursor_percentage: f32,
}

impl CaptionClipper {
    pub fn new() -> Self {
        Self {
            visible_slice_first_overflow: None,
            visible_slice: IdxSlice::new(0),
            prev_cap_len: 0,
            visible_text: String::new(),
            visible_text_dx: vec![],
            scroll_cursor: 0,
            scroll_cursor_percentage: 0.0,
            can_off_cursor: None,
        }
    }

    pub fn visible_slice(&self) -> &IdxSlice {
        &self.visible_slice
    }

    fn text_length_unchanged(&self, text: &str) -> bool {
        text.len() == self.prev_cap_len
    }

    pub fn clip_text<'a>(
        &'a mut self,
        text: &'a str,
        font_size: f32,
        frame_bounds: Vec2<f32>,
        text_writer: &TextWriter,
        margin_right: f32,
    ) -> &'a str {
        if text.is_empty() == true {
            self.scroll_cursor = 0;
            self.visible_slice_first_overflow = None;
            return "";
        }

        if self.text_length_unchanged(text) {
            //use cached results if caption hasn't changed
            return &self.visible_text;
        }

        self.clear();

        self.process_request_scroll_cursor_offset(
            text,
            text_writer,
            frame_bounds,
            margin_right,
            font_size,
        );

        let text_size = font_size;
        let visible_slice = &mut self.visible_slice;
        let visible_slice_first_overflow = &mut self.visible_slice_first_overflow;

        *visible_slice = IdxSlice::default();
        // *visible_slice_first_overflow = None;
        self.scroll_cursor_percentage = 0.0;

        let mut clipped_text;
        let mut aabb;
        let byte_slice = text.as_bytes();
        let num_bytes = byte_slice.len() as isize;
        let cursor_ubound = (num_bytes - self.scroll_cursor).clamp(0, num_bytes) as usize;
        let max_text_width = (frame_bounds.x() - margin_right).max(0.0);

        for _ in &byte_slice[..cursor_ubound] {
            clipped_text = visible_slice.get_slice(text);
            aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, text_size);

            if aabb.w < max_text_width {
                visible_slice.push_rear(1);
            } else {
                //on the first overflow record text positions
                if visible_slice_first_overflow.is_none() {
                    *visible_slice_first_overflow = Some(*visible_slice);
                }

                visible_slice.shift(1);
            }
        }

        while {
            clipped_text = visible_slice.get_slice(text);
            aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, text_size);
            aabb.w >= max_text_width
        } {
            visible_slice.pop_front(1);
        }

        clipped_text = visible_slice.get_slice(text);

        //update visible_text string
        self.visible_text.clear();
        self.visible_text.push_str(clipped_text);

        //assign new previous text length
        self.prev_cap_len = text.len();

        self.recompute_visible_character_widths(text_writer, text, font_size);
        self.recompute_scroll_cursor_percentage(text);
        clipped_text
    }

    fn recompute_scroll_cursor_percentage(&mut self, text: &str) {
        /******************************************************
        Computing scroll_cursor_percentage, visually because
        the equation is kinda hard to come up with in my head
        -------------------------------------------------------
        Legend
        -------------------------------------------------------
        t = text
        iof = initial overflow index slice
        cub = cursor upper bound
        -------------------------------------------------------
        Example
        -------------------------------------------------------
        t  :a b c d e f g h i j k l m n o p q r s t u v w x y z
        iof:^           ^
        cub:                              ^
        -------------------------------------------------------
        based on diagram it looks like equation is:
        -------------------------------------------------------
        percentage = (cub-iof.end)/(t.len() - iof.len())
        ******************************************************/
        let num_bytes = text.as_bytes().len() as isize;
        let cursor_ubound = (num_bytes - self.scroll_cursor).clamp(0, num_bytes) as usize;
        if let Some(slice) = self.visible_slice_first_overflow {
            self.scroll_cursor_percentage = (cursor_ubound as isize - slice.as_range().end as isize)
                .max(0) as f32
                / (self.prev_cap_len as isize - slice.len() as isize).max(1) as f32;
            self.scroll_cursor_percentage = self.scroll_cursor_percentage.clamp(0.0, 1.0);
            // println!("percentage = {}",self.scroll_cursor_percentage);
        }
    }

    fn recompute_visible_character_widths(
        &mut self,
        text_writer: &TextWriter,
        _text: &str,
        font_size: f32,
    ) {
        let mut prev_w = 0.0;
        let clipped_text = self.visible_text.as_str();
        self.visible_text_dx.clear();

        for k in 1..=clipped_text.len() {
            let cumulative_text = &clipped_text[..k];
            let cur_w = text_writer
                .calc_text_aabb(cumulative_text, 0.0, 0.0, font_size)
                .w;
            self.visible_text_dx.push(cur_w - prev_w);
            prev_w = cur_w;
        }

        // // view clipped_text info
        // println!(
        //     "{clipped_text},\n{:?}\n{:?}\n{}\n",
        //     self.visible_text_dx,
        //     self.visible_slice,
        //     self.visible_slice.get_slice(_text)
        // );
    }

    pub fn set_scroll_cursor_by_percentage(&mut self, new_percentage: f32) {
        let num_bytes = self.prev_cap_len as isize;
        if let Some(slice) = self.visible_slice_first_overflow {
            let new_cursor_ubound = new_percentage * (self.prev_cap_len - slice.len()) as f32
                + slice.as_range().end as f32;

            let new_cursor = num_bytes - new_cursor_ubound as isize;
            self.scroll_cursor = new_cursor.clamp(1, num_bytes);
            self.scroll_cursor_percentage = new_percentage;

            //done to envoke recomputation
            self.prev_cap_len += 1;
        }
    }

    pub fn get_scroll_cursor_percentage(&self) -> f32 {
        self.scroll_cursor_percentage
    }

    fn process_request_scroll_cursor_offset(
        &mut self,
        text: &str,
        text_writer: &TextWriter,
        frame_bounds: Vec2<f32>,
        margin_right: f32,
        font_size: f32,
    ) {
        const MARGIN_SCALING_TO_MAKE_SURE_CURSOR_REACES_THE_START_OF_THE_TEXT: f32 = 1.5;

        if let Some(off) = self.can_off_cursor.take() {
            let byte_slice = text.as_bytes();
            let num_bytes = byte_slice.len() as isize;
            let cursor_ubound =
                (num_bytes - (self.scroll_cursor + off)).clamp(0, num_bytes) as usize;
            let clipped_text = &text[..cursor_ubound];
            let clipped_text_aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, font_size);
            let is_overflow_on_x = || {
                let clipped_max_width = frame_bounds.x()
                    - margin_right
                        * MARGIN_SCALING_TO_MAKE_SURE_CURSOR_REACES_THE_START_OF_THE_TEXT;
                clipped_text_aabb.w > clipped_max_width
            };

            if is_overflow_on_x() || off <= 0 {
                self.scroll_cursor += off;
                self.scroll_cursor = self.scroll_cursor.clamp(0, num_bytes);
            }
        }
    }
    pub fn get_text_postion_given_horizontal_disp(&self, disp_x: f32) -> usize {
        let mut total_len = 0.0;
        let mut local_idx = 0;
        let mut global_idx = self.visible_slice.lbound();

        // let visible_text_len = self.visible_text.len();
        while total_len < disp_x && local_idx < self.visible_text_dx.len() {
            total_len += self.visible_text_dx[local_idx];
            local_idx += 1;
            global_idx += 1;
        }
        // println!(
        //     "c = {}",
        //     self.visible_text.as_bytes()[local_idx.min(visible_text_len - 1)] as char
        // );
        global_idx
    }

    pub fn get_visible_cursor_displacement(&self, global_text_index: usize) -> f32 {
        self.visible_text_dx
            .iter()
            .take(global_text_index - self.visible_slice.lbound())
            .fold(0.0, |acc, &e| acc + e)
    }

    pub fn request_offset_of_scroll_cursor(&mut self, off: isize) {
        self.can_off_cursor = Some(off);
        //to avoid caching
        self.prev_cap_len += 1;
    }

    fn clear(&mut self) {
        self.prev_cap_len = 0;
        self.visible_text.clear();
        self.visible_text_dx.clear();
        // self.visible_slice_first_overflow = None;
    }
}

pub struct TextBoxState {
    pub frame: FrameState,
    aligner: TextAligner2D,
    text: String,
    text_size: f32,
    text_cursor: usize,
    clipper: CaptionClipper,
    text_area: AABB2<f32>,
    cursor_area: AABB2<f32>,
    t0: Instant,
}

impl TextBoxState {
    pub fn new() -> Self {
        Self {
            frame: FrameState::new(),
            aligner: TextAligner2D::new(),
            text: String::new(),
            text_size: 12.0,
            clipper: CaptionClipper::new(),
            cursor_area: AABB2::zero(),
            text_area: AABB2::zero(),
            text_cursor: 0,
            t0: Instant::now(),
        }
    }
}

impl TextBoxState {
    pub fn update_char_cursor(&mut self, mouse_position: Vec2<f32>) {
        let relative_horizontal_postion = (mouse_position - self.text_area.min_pos).x();
        let new_text_cursor_position = self
            .clipper
            .get_text_postion_given_horizontal_disp(relative_horizontal_postion);
        self.text_cursor = new_text_cursor_position;
    }

    pub fn offset_cursor(&mut self, off: isize) {
        self.text_cursor =
            (self.text_cursor as isize + off).clamp(0, self.text.len() as isize) as usize;
    }

    pub fn push_char_at_cursor(&mut self, c: char) {
        self.text_cursor = self.text_cursor.clamp(0, self.text.len());
        self.text.insert(self.text_cursor, c);
        self.text_cursor += 1;
    }

    pub fn remove_char_at_cursor(&mut self) {
        if self.text_cursor > 0 && self.text_cursor <= self.text.len() {
            self.text.remove(self.text_cursor - 1);
            self.text_cursor -= 1;
        }
    }
}

impl GuiComponent for TextBoxState {
    fn common(&self) -> &GuiCommonState {
        self.frame.common()
    }

    fn common_mut(&mut self) -> &mut GuiCommonState {
        self.frame.common_mut()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn render_entry<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
    ) {
        const HORIZONTAL_MARGIN: f32 = 20.0;
        let win_w = state.win_w;
        let win_h = state.win_h;

        self.frame.render_entry(gl, state.clone(), text_writer);

        layer_lock(gl, state.level, *self.flags());

        let &old_sf = text_writer.horizontal_scaling_factor();
        *text_writer.horizontal_scaling_factor_mut() = 1.3;

        let clipper = &mut self.clipper;
        let caption = &self.text;
        let frame_bounds = self.frame.bounds();
        let text_size = self.text_size;

        let scroll_percentage = clipper.get_scroll_cursor_percentage();
        let clipped_text = clipper.clip_text(
            caption,
            self.text_size,
            frame_bounds,
            text_writer,
            HORIZONTAL_MARGIN * 2.0,
        );
        let position = state.global_position;

        if clipped_text.is_empty() == false {
            let text_aabb = text_writer.calc_text_aabb(clipped_text, 0.0, 0.0, text_size);

            let aligned_global_position = self.aligner.compute_position(
                Vec2::convert(position),
                Vec2::from([text_aabb.w, text_aabb.h]),
                self.frame.bounds(),
            );

            text_writer.draw_text_line(
                clipped_text,
                aligned_global_position.x() + HORIZONTAL_MARGIN,
                aligned_global_position.y(),
                text_size,
                Some((win_w as u32, win_h as u32)),
            );

            self.text_area = AABB2::from_point_and_lengths(
                Vec2::from([
                    aligned_global_position.x() + HORIZONTAL_MARGIN,
                    aligned_global_position.y(),
                ]),
                Vec2::from([text_aabb.w, text_aabb.h]),
            );

            unsafe {
                //re-enable
                gl.enable(glow::BLEND);
            }

            //draw scroll bar
            let scroll_bar_bounds = Vec2::from([50.0, 12.0]);
            let scroll_bar_pos = [
                (aligned_global_position.x() + HORIZONTAL_MARGIN)
                    + (text_aabb.w - scroll_bar_bounds.x()) * scroll_percentage,
                aligned_global_position.y() + text_aabb.h,
            ];

            state
                .renderer
                .builder(gl, GuiShaderKind::RoundedBox)
                .set_window(win_w, win_h)
                .set_position(scroll_bar_pos, Vec4::convert(scroll_bar_bounds))
                .set_background_color(Vec4::rgb_u32(!0))
                .set_edge_color(Vec4::rgb_u32(0x000000))
                .set_edge_thickness(0.01)
                .set_roundness_vec([1., 1., 10.0, 10.0])
                .set_bounds(scroll_bar_bounds)
                .render();

            //update cursor area
            self.cursor_area = AABB2::from_segment(
                Vec2::from([
                    (aligned_global_position.x() + HORIZONTAL_MARGIN)
                        + (text_aabb.w - scroll_bar_bounds.x()) * 0.0,
                    aligned_global_position.y() + text_aabb.h * 1.0,
                ]),
                Vec2::from([
                    (aligned_global_position.x() + HORIZONTAL_MARGIN)
                        + (text_aabb.w - scroll_bar_bounds.x()) * 1.0,
                    (aligned_global_position.y() + text_aabb.h * 1.5)
                        .min(position.y() + self.frame.bounds().y()),
                ]),
            );

            unsafe {
                gl.blend_func(glow::ONE, glow::ONE);
            }
            // // render cursor_area bounding box
            // state
            //     .renderer
            //     .builder(gl, GuiShaderKind::RoundedBox)
            //     .set_window(win_w, win_h)
            //     .set_position(
            //         Vec4::to_pos(self.cursor_area.s0),
            //         Vec4::convert(self.cursor_area.dims()),
            //     )
            //     .set_background_color(Vec4::rgb_u32(0))
            //     .set_edge_color(Vec4::rgb_u32(!0))
            //     .set_roundness_vec([1.; 4])
            //     .set_bounds(self.cursor_area.dims())
            //     .render();

            // // render text_area bounding box
            // state
            //     .renderer
            //     .builder(gl, GuiShaderKind::RoundedBox)
            //     .set_window(win_w, win_h)
            //     .set_position(
            //         Vec4::to_pos(self.text_area.s0),
            //         Vec4::convert(self.text_area.dims()),
            //     )
            //     .set_background_color(Vec4::rgb_u32(0))
            //     .set_edge_color(Vec4::rgb_u32(!0))
            //     .set_roundness_vec([1.; 4])
            //     .set_bounds(self.text_area.dims())
            //     .render();

            // render cursor bounding box
            let is_text_cursor_visible = self
                .clipper
                .visible_slice
                .is_in_range_include_upper_bound(self.text_cursor);

            let elapsed_time_ms = self.t0.elapsed().as_millis();
            //blink every 1024ms
            let cursor_blink_index = elapsed_time_ms >> 10;

            if is_text_cursor_visible && cursor_blink_index % 2 == 0 {
                let visible_cursor_displacement = self
                    .clipper
                    .get_visible_cursor_displacement(self.text_cursor);

                let cursor_pos = Vec2::from([
                    aligned_global_position.x() + HORIZONTAL_MARGIN + visible_cursor_displacement,
                    aligned_global_position.y(),
                ]);

                let cursor_bounds = Vec2::from([2.0, text_aabb.h]);
                state
                    .renderer
                    .builder(gl, GuiShaderKind::Rectangle)
                    .set_window(win_w, win_h)
                    .set_position(Vec4::convert(cursor_pos), Vec4::convert(cursor_bounds))
                    .set_background_color(Vec4::rgb_u32(0xff0000))
                    .set_edge_color(Vec4::rgb_u32(0xff0000))
                    .set_roundness_vec([1.; 4])
                    .set_bounds(cursor_bounds)
                    .render();
            }

            unsafe {
                gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            }

            layer_unlock(gl);

            //restore previous sf
            *text_writer.horizontal_scaling_factor_mut() = old_sf;
        }
    }

    fn render_exit<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
    ) {
        /* not implemented on purpose  */
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
        let textbox_key =
            unsafe { manager.add_component_deferred(GuiComponentKey::default(), None) };
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
        self.state.as_mut().unwrap().set_bounds(bounds);
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
        self.state
            .as_mut()
            .unwrap()
            .set_rel_position(Vec2::from(pos));
        self
    }

    pub fn with_visibility(mut self, visibility: bool) -> Self {
        self.state.as_mut().unwrap().frame.set_visible(visibility);
        self
    }

    pub fn with_alignment(mut self, alignment: [TextAlignment; 2]) -> Self {
        self.state.as_mut().unwrap().aligner.alignment_mode_per_axis = alignment;
        self
    }

    pub fn with_caption(mut self, caption: String) -> Self {
        self.state.as_mut().unwrap().text = caption;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.state.as_mut().unwrap().text_size = size;
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

    fn state(&mut self) -> &mut Option<Self::ComponentKind> {
        &mut self.state
    }

    fn build(mut self) -> GuiComponentKey {
        // add default event listeners
        self = self
            .with_listener(GuiEventKind::OnMouseDown, |tb, ek, _mrc| {
                if let EventKind::MouseDown { x, y, .. } = ek {
                    let mouse_pos = Vec2::from([x as f32, y as f32]);

                    if tb.cursor_area.is_point_inside(mouse_pos) {
                        let dims = tb.cursor_area.dims();
                        let new_percentage = (x as f32 - tb.cursor_area.min_pos.x()) / dims.x();
                        const SNAP_TO_BOUNDS_THRESH: f32 = 0.1;
                        if new_percentage < SNAP_TO_BOUNDS_THRESH {
                            tb.clipper.request_offset_of_scroll_cursor(1);
                        }
                        if new_percentage > (1. - SNAP_TO_BOUNDS_THRESH) {
                            tb.clipper.request_offset_of_scroll_cursor(-1);
                        }

                        tb.clipper.request_offset_of_scroll_cursor(0);
                        tb.clipper.set_scroll_cursor_by_percentage(new_percentage);
                    } else if tb.text_area.is_point_inside(mouse_pos) {
                        tb.update_char_cursor(mouse_pos);
                    }
                }
            })
            .with_listener_advanced(
                GuiEventKind::OnDrag,
                Box::new(|info| {
                    if let EventKind::MouseMove { x, y, .. } = info.event {
                        let tb_key = info.key;
                        let gui_comp_tree = info.gui_comp_tree;
                        let tb = gui_comp_tree
                            .get_mut(tb_key)
                            .expect("tb_key should be valid")
                            .as_any_mut()
                            .downcast_mut::<TextBoxState>()
                            .unwrap();
                        let mouse_pos = Vec2::from([x as f32, y as f32]);
                        if tb.cursor_area.is_point_inside(mouse_pos) {
                            let dims = tb.cursor_area.dims();
                            let new_percentage = (x as f32 - tb.cursor_area.min_pos.x()) / dims.x();
                            tb.clipper.set_scroll_cursor_by_percentage(new_percentage);
                        }
                    }
                    None
                }),
            )
            .with_listener(GuiEventKind::OnWheelWhileFocused, |tb, e, _mrc| {
                let wheel_dir = e.wheel();
                tb.clipper
                    .request_offset_of_scroll_cursor(wheel_dir as isize);
            })
            .with_listener(GuiEventKind::OnKeyDown, |comp, e, _mrq| {
                if let EventKind::KeyDown { code } = e {
                    match code {
                        KeyCode::BACKSPACE => {
                            comp.remove_char_at_cursor();
                        }
                        KeyCode::ARROW_LEFT => {
                            comp.offset_cursor(-1);
                        }
                        KeyCode::ARROW_RIGHT => {
                            comp.offset_cursor(1);
                        }
                        KeyCode::SHIFT_L | KeyCode::SHIFT_R | KeyCode::CTRL_L | KeyCode::CTRL_R => {
                        }
                        _ => {
                            let c = code.key_val().unwrap_or_default();
                            if c.is_ascii() {
                                comp.push_char_at_cursor(c);
                            }
                        }
                    }
                }
            });

        let manager = self.manager;
        let textbox_parent_node_id = self.parent_key.unwrap_or_default();
        let textbox_node_id = self.textbox_key.expect("textbox key missing");
        let textbox_state = self.state.expect("textbox state missing");

        // set node state
        *manager.gui_component_tree.get_mut_uninit(textbox_node_id) =
            MaybeUninit::new(Box::new(textbox_state));

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
