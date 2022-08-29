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
    fn is_visible(&self) -> bool {
        self.slider_frame.is_visible
    }
    fn set_visible(&mut self, is_visible: bool) {
        self.slider_frame.is_visible = is_visible;
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
        &mut self,
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

    pub fn with_button_listener<CB>(self, kind: GuiEventKind, mut cb: CB) -> Self
    where
        CB: FnMut(&mut FrameState, EventKind, &ProgramState) + 'static,
    {
        let slider_button_key = self.slider_button_key.expect("slider key not found");
        self.manager.push_listener(
            slider_button_key,
            ComponentEventListener::new(
                kind,
                Box::new(move |info| {
                    let state = info.state;
                    let slider_button_key = info.key;
                    let event = info.event;

                    let slider_button_state = info
                        .gui_comp_tree
                        .get_mut(slider_button_key)?
                        .as_any_mut()
                        .downcast_mut::<FrameState>()?;

                    cb(slider_button_state, event, state);

                    None
                }),
            ),
        );
        self
    }
}

impl<'a, ProgramState> HasComponentBuilder<ProgramState> for SliderBuilder<'a, ProgramState> {
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

            //used to clamp and verticially center slider_button
            let slider_frame_bounds = slider_frame_state.get_bounds();
            let mut slider_button_bounds = slider_button_state.get_bounds();

            //this clamps max height of button to be the parents height
            slider_button_bounds[1] = slider_button_bounds[1].clamp(0.0, slider_frame_bounds[1]);

            //assign newly clamped bounds to state
            slider_button_state.bounds[1] = slider_button_bounds[1];

            //center slider
            slider_button_state.rel_pos[0] = 0.0;
            slider_button_state.rel_pos[1] =
                (slider_frame_bounds[1] - slider_button_bounds[1]) * 0.5;

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
                    let max_horizontal_rel_pos = frame_bounds[0] - button_bounds[0];
                    let button_slider_pos = *tree
                        .get(slider_button_key)
                        .expect("slider_button_key invalid")
                        .rel_position();

                    let vertically_centered_and_horizontally_clamped_relative_position =
                        Vec2::from([
                            button_slider_pos[0].clamp(0.0, max_horizontal_rel_pos),
                            frame_bounds[1] * 0.5 - button_bounds[1] * 0.5,
                        ]);

                    //set newly computed position
                    tree.get_mut(slider_button_key)
                        .expect("slider_button_key invalid")
                        .set_rel_position(
                            vertically_centered_and_horizontally_clamped_relative_position,
                        );

                    //update percentage
                    let new_percentage =
                        (button_slider_pos.x() / max_horizontal_rel_pos).clamp(0.0, 1.0);
                    tree.get_mut(slider_frame_key)
                        .expect("slider_frame_key invalid")
                        .as_any_mut()
                        .downcast_mut::<Self::ComponentKind>()
                        .expect("slider_frame_key should alias SliderState")
                        .percentage = new_percentage;

                    None
                }),
            ),
        );

        self.manager.push_listener(
            slider_frame_key,
            ComponentEventListener::new(
                GuiEventKind::OnWheelWhileHovered,
                Box::new(|info| {
                    let wheel = info.event.wheel();

                    let tree = info.gui_comp_tree;

                    let slider_frame_key = info.key;

                    let slider_button_key = tree
                        .get_mut(slider_frame_key)
                        .unwrap()
                        .as_any_mut()
                        .downcast_mut::<SliderState>()
                        .unwrap()
                        .slider_button_key;

                    let frame_bounds = tree
                        .get(slider_frame_key)
                        .expect("slider_frame component")
                        .get_bounds();

                    let button_bounds = tree
                        .get(slider_button_key)
                        .expect("slider_button component")
                        .get_bounds();

                    //move slider button,horizontally by increments of 5% of the parent bounds width
                    let wheel_dx = (frame_bounds[0] * 0.05) * wheel;

                    //translate the slider like normal
                    tree.get_mut(slider_button_key)
                        .expect("slider_button")
                        .translate(Vec2::from([wheel_dx, 0.0]));

                    //vertically center the slider on drag
                    let max_horizontal_rel_pos = frame_bounds[0] - button_bounds[0];
                    let button_slider_pos = *tree
                        .get(slider_button_key)
                        .expect("slider_button_key invalid")
                        .rel_position();

                    let vertically_centered_and_horizontally_clamped_relative_position =
                        Vec2::from([
                            button_slider_pos.x().clamp(0.0, max_horizontal_rel_pos),
                            frame_bounds.y() * 0.5 - button_bounds.y() * 0.5,
                        ]);

                    //set newly computed position
                    tree.get_mut(slider_button_key)
                        .expect("slider_button_key invalid")
                        .set_rel_position(
                            vertically_centered_and_horizontally_clamped_relative_position,
                        );

                    //update percentage
                    let new_percentage =
                        (button_slider_pos.x() / max_horizontal_rel_pos).clamp(0.0, 1.0);
                    tree.get_mut(slider_frame_key)
                        .expect("slider_frame_key invalid")
                        .as_any_mut()
                        .downcast_mut::<Self::ComponentKind>()
                        .expect("slider_frame_key should alias SliderState")
                        .percentage = new_percentage;

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
