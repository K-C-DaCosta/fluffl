use super::*;

pub enum TextSize {
    Fixed(f32),
    Dynamic,
}

pub struct LabelState {
    common: GuiCommonState,
    aligner: TextAligner2D,
    caption: String,
    size: TextSize,
}

impl Default for LabelState {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelState {
    pub fn new() -> Self {
        Self {
            common: GuiCommonState::new().with_flags(component_flags::VISIBLE),
            caption: String::new(),
            size: TextSize::Dynamic,
            aligner: TextAligner2D::new(),
        }
    }
}

impl GuiComponent for LabelState {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn common(&self) -> &GuiCommonState {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GuiCommonState {
        &mut self.common
    }

    fn render_entry<'a>(
        &mut self,
        gl: &GlowGL,
        state: RenderState<'a>,
        text_writer: &mut TextWriter,
    ) {
        let win_w = state.win_w;
        let win_h = state.win_h;
        layer_lock(gl, state.level, *self.flags());

        let text_height = match self.size {
            TextSize::Dynamic => self.bounds().y(),
            TextSize::Fixed(height) => height,
        };

        let text_bounds = text_writer.calc_text_aabb(&self.caption, 0.0, 0.0, text_height);
        let text_bounds = Vec2::from([text_bounds.w(), text_bounds.h()]);

        let aligned_position = self.aligner.compute_position(
            Vec2::convert(state.global_position),
            text_bounds,
            self.bounds(),
        );

        text_writer.draw_text_line(
            &self.caption,
            aligned_position.x(),
            aligned_position.y(),
            text_height,
            Some((win_w as u32, win_h as u32)),
        );

        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        layer_unlock(gl);
    }

    fn render_exit<'a>(
        &mut self,
        _gl: &GlowGL,
        _state: RenderState<'a>,
        _text_writer: &mut TextWriter,
    ) {
        /* not implemented on purpose */
    }
}

pub struct LabelBuilder<'a, ProgramState> {
    label_key: Option<GuiComponentKey>,
    parent: Option<GuiComponentKey>,
    state: Option<LabelState>,
    manager: &'a mut GuiManager<ProgramState>,
}

impl<'a, ProgramState> LabelBuilder<'a, ProgramState> {
    pub fn new(manager: &'a mut GuiManager<ProgramState>) -> Self {
        let label_key =
            Some(unsafe { manager.add_component_deferred(GuiComponentKey::default(), None) });
        Self {
            label_key,
            parent: Some(GuiComponentKey::default()),
            state: Some(LabelState::new()),
            manager,
        }
    }

    pub fn with_position<T: Into<Vec2<f32>>>(mut self, pos: T) -> Self {
        self.state.as_mut().unwrap().set_rel_position(pos.into());
        self
    }

    pub fn with_bounds<T: Into<Vec2<f32>>>(mut self, bounds: T) -> Self {
        self.state.as_mut().unwrap().set_bounds(bounds.into());
        self
    }

    pub fn with_caption<T: AsRef<str>>(mut self, text: T) -> Self {
        self.state.as_mut().unwrap().caption.clear();
        self.state.as_mut().unwrap().caption.push_str(text.as_ref());
        self
    }

    pub fn with_alignment_horizontal(self, mode: TextAlignment) -> Self {
        self.with_alignment(mode, 0)
    }

    pub fn with_alignment_vertical(self, mode: TextAlignment) -> Self {
        self.with_alignment(mode, 1)
    }

    fn with_alignment(mut self, mode: TextAlignment, axis: usize) -> Self {
        self.state.as_mut().unwrap().aligner.alignment_mode_per_axis[axis] = mode;
        self
    }

    pub fn with_text_size(mut self, size: TextSize) -> Self {
        self.state.as_mut().unwrap().size = size;
        self
    }
}

impl<'a, ProgramState> HasComponentBuilder<ProgramState> for LabelBuilder<'a, ProgramState> {
    type ComponentKind = LabelState;

    fn key(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.label_key
    }

    fn manager(&mut self) -> &mut GuiManager<ProgramState> {
        self.manager
    }

    fn parent(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.parent
    }

    fn state(&mut self) -> &mut Option<Self::ComponentKind> {
        &mut self.state
    }

    fn build(mut self) -> GuiComponentKey {
        let label_key = self.label_key.expect("label key should always exist");
        let parent = self.parent.unwrap_or_default();
        let label_state = self.state.take().expect("label state should exist");

        let gui_component_tree = &mut self.manager.gui_component_tree;
        gui_component_tree.set_parent(label_key, parent);

        *gui_component_tree.get_mut_uninit(label_key) = MaybeUninit::new(Box::new(label_state));
        gui_component_tree.reconstruct_preorder();

        label_key
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn builder_label(&mut self) -> LabelBuilder<'_, ProgramState> {
        LabelBuilder::new(self)
    }
}
