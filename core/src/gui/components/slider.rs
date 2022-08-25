use super::*;

pub struct SliderState {
    pub slider_button_key: GuiComponentKey,
    pub slider_frame: FrameState,
    pub percentage: f32,
}

impl SliderState {
    fn new() -> Self {
        Self {
            slider_button_key: GuiComponentKey::default(),
            slider_frame: FrameState::new(),
            percentage: 0.0,
        }
    }
}

impl GuiComponent for SliderState {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn rel_position(&self) -> &Vec2<f32> {
        &self.slider_frame.rel_pos
    }

    fn set_rel_position(&mut self, pos: Vec2<f32>) {
        self.slider_frame.rel_pos = pos;
    }

    fn get_bounds(&self) -> Vec2<f32> {
        self.slider_frame.bounds
    }

    fn set_bounds(&mut self, bounds: Vec2<f32>) {
        self.slider_frame.bounds = bounds;
    }

    fn render<'a>(
        &self,
        gl: &GlowGL,
        state: RenderState<'a>,
        _text_writer: &mut TextWriter,
        win_w: f32,
        win_h: f32,
    ) {
        state
            .renderer
            .builder(gl, GuiShaderKind::RoundedBox)
            .set_window(win_w, win_h)
            .set_background_color(self.slider_frame.color)
            .set_bounds(self.slider_frame.bounds)
            .set_edge_color(self.slider_frame.edge_color)
            .set_roundness_vec(self.slider_frame.roundness)
            .set_position(
                state.global_position,
                Vec4::convert(self.slider_frame.bounds),
            )
            .render();
    }
}

pub struct SliderBuilder<'a, ProgramState> {
    manager: &'a mut GuiManager<ProgramState>,
    slider_frame_state: Option<SliderState>,
    slider_frame_key: Option<GuiComponentKey>,
    slider_button_state: Option<FrameState>,
    slider_button_key: Option<GuiComponentKey>,
    parent: Option<GuiComponentKey>,
    key: Option<GuiComponentKey>,
}

impl<'a, ProgramState> SliderBuilder<'a, ProgramState> {
    pub fn new(manager: &'a mut GuiManager<ProgramState>) -> Self {
        let slider_frame_key =
            Some(manager.add_component_deferred(GuiComponentKey::default(), None));
        let slider_button_key =
            Some(manager.add_component_deferred(GuiComponentKey::default(), None));

        let mut slider_state = SliderState::new();
        slider_state.slider_button_key = slider_button_key.unwrap();

        Self {
            manager,
            parent: None,
            slider_frame_state: Some(slider_state),
            slider_button_state: Some(FrameState::new()),
            slider_frame_key,
            slider_button_key,
            key: slider_frame_key,
        }
    }

    pub fn with_bounds<T: Into<Vec2<f32>>>(mut self, bounds: T) -> Self {
        self.slider_frame_state
            .as_mut()
            .unwrap()
            .slider_frame
            .bounds = bounds.into();
        self
    }

    pub fn with_percentage(mut self, percentage: f32) -> Self {
        self.slider_frame_state.as_mut().unwrap().percentage = percentage;
        self
    }

    pub fn with_position<T: Into<Vec2<f32>>>(mut self, rel_pos: T) -> Self {
        self.slider_frame_state
            .as_mut()
            .unwrap()
            .slider_frame
            .rel_pos = rel_pos.into();
        self
    }

    pub fn with_color<T: Into<Vec4<f32>>>(mut self, color: T) -> Self {
        self.slider_frame_state.as_mut().unwrap().slider_frame.color = color.into();
        self
    }

    pub fn with_edge_color<T: Into<Vec4<f32>>>(mut self, color: T) -> Self {
        self.slider_frame_state
            .as_mut()
            .unwrap()
            .slider_frame
            .edge_color = color.into();
        self
    }

    pub fn with_roundness<T: Into<Vec4<f32>>>(mut self, roundness: T) -> Self {
        self.slider_frame_state
            .as_mut()
            .unwrap()
            .slider_frame
            .roundness = roundness.into();
        self
    }

    pub fn with_button_bounds<T: Into<Vec2<f32>>>(mut self, bounds: T) -> Self {
        self.slider_button_state.as_mut().unwrap().bounds = bounds.into();
        self
    }

    pub fn with_button_color<T: Into<Vec4<f32>>>(mut self, color: T) -> Self {
        self.slider_button_state.as_mut().unwrap().color = color.into();
        self
    }

    pub fn with_button_edge_color<T: Into<Vec4<f32>>>(mut self, color: T) -> Self {
        self.slider_button_state.as_mut().unwrap().edge_color = color.into();
        self
    }

    pub fn with_button_roundness<T: Into<Vec4<f32>>>(mut self, roundness: T) -> Self {
        self.slider_button_state.as_mut().unwrap().roundness = roundness.into();
        self
    }
}

impl<'a, ProgramState> HasBuilder<ProgramState> for SliderBuilder<'a, ProgramState> {
    type ComponentKind = SliderState;

    fn key(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.key
    }

    fn manager(&mut self) -> &mut GuiManager<ProgramState> {
        self.manager
    }

    fn parent(&mut self) -> &mut Option<GuiComponentKey> {
        &mut self.parent
    }

    fn build(mut self) -> GuiComponentKey {
        let slider_frame_parent = self.parent.unwrap_or_default();
        let slider_frame_key = self.slider_frame_key.expect("slider_frame_key not set");
        let slider_button_key = self.slider_button_key.expect("slider_button_key");

        /*scope block so that all variables inside go out of scope when finished*/
        {
            let slider_frame_state = self
                .slider_frame_state
                .take()
                .expect("slider_frame_state not found");

            let mut slider_button_state = self
                .slider_button_state
                .take()
                .expect("slider button state not found");

            //center slider
            let slider_frame_bounds = slider_frame_state.get_bounds();
            let slider_button_bounds = slider_button_state.get_bounds();
            slider_button_state.rel_pos[0] = 0.0;
            slider_button_state.rel_pos[1] =
                slider_frame_bounds[1] * 0.5 - slider_button_bounds[1] * 0.5;

            //finally write components to the deferred tree nodes
            *self
                .manager
                .gui_component_tree
                .get_mut_opt(slider_frame_key) = Some(Box::new(slider_frame_state));

            *self
                .manager
                .gui_component_tree
                .get_mut_opt(slider_button_key) = Some(Box::new(slider_button_state));

            //set frame parent
            self.manager
                .gui_component_tree
                .set_parent(slider_frame_key, slider_frame_parent);

            //frame should be parent of button
            self.manager
                .gui_component_tree
                .set_parent(slider_button_key, slider_frame_key);
        }

        self.manager.push_listener(
            slider_button_key,
            ComponentEventListener::new(
                GuiEventKind::OnDrag,
                Box::new(|info| {
                    let disp = info.event.disp();
                    let tree = info.gui_comp_tree;
                    let slider_button_key = info.key;
                    let slider_frame_key = tree
                        .get_parent_id(slider_button_key)
                        .expect("slider_button SHOULD have a parent");

                    let frame_bounds = tree
                        .get(slider_frame_key)
                        .expect("slider_frame component")
                        .get_bounds();

                    let button_bounds = tree
                        .get(slider_button_key)
                        .expect("slider_button component")
                        .get_bounds();

                    //translate the slider like normal
                    tree.get_mut(slider_button_key)
                        .expect("slider_button")
                        .translate(disp);

                    //vertically center the slider on drag
                    let button_slider_pos = *tree.get(slider_button_key).unwrap().rel_position();
                    let vertically_centered_relative_position = Vec2::from([
                        button_slider_pos[0].clamp(0.0, frame_bounds[0] - button_bounds[0]),
                        frame_bounds[1] * 0.5 - button_bounds[1] * 0.5,
                    ]);

                    tree.get_mut(slider_button_key)
                        .unwrap()
                        .set_rel_position(vertically_centered_relative_position);

                    None
                }),
            ),
        );

        //pretty much all build implementations will have this
        self.manager.gui_component_tree.reconstruct_preorder();
        slider_frame_key
    }
}

impl<ProgramState> GuiManager<ProgramState> {
    pub fn builder_slider(&mut self) -> SliderBuilder<ProgramState> {
        SliderBuilder::new(self)
    }
}
