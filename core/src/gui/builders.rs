use super::*;

pub struct ComponentBuilder<'a, CompKind, ProgramState> {
    manager: &'a mut GuiManager<ProgramState>,
    component: Option<CompKind>,
    parent: Option<GuiComponentKey>,
    key: Option<GuiComponentKey>,
}

impl<'a, Component, ProgramState> ComponentBuilder<'a, Component, ProgramState>
where
    Component: GuiComponent + 'static,
{
    pub fn new(manager: &'a mut GuiManager<ProgramState>) -> Self {
        Self {
            manager,
            component: None,
            parent: None,
            key: None,
        }
    }

    fn create_key_if_possible(&mut self) {
        match (self.key, self.component.take()) {
            (Some(_), _) => (),
            (None, Some(comp)) => {
                self.key = Some(
                    self.manager
                        .add_component(self.parent.unwrap_or_default(), Box::new(comp)),
                );
            }
            (None, None) => {
                panic!("component not set, build failed")
            }
        }
    }

    pub fn with_component(mut self, comp: Component) -> Self {
        self.component = Some(comp);
        self
    }

    pub fn with_parent(mut self, parent: GuiComponentKey) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_listener<Listener>(self, kind: GuiEventKind, mut listener: Listener) -> Self
    where
        Listener: FnMut(&mut Component, &ProgramState, EventKind) -> Option<()> + 'static,
    {
        self.with_listener_advanced(
            kind,
            Box::new(move |event_state| {
                let state = event_state.state;
                let event = event_state.event;
                let key = event_state.key;
                let comp = event_state
                    .gui_comp_tree
                    .get_mut(key)?
                    .as_any_mut()
                    .downcast_mut::<Component>()?;
                listener(comp, state, event)
            }),
        )
    }

    pub fn with_listener_advanced(
        mut self,
        kind: GuiEventKind,
        cb: ListenerCallBack<ProgramState>,
    ) -> Self {
        self.create_key_if_possible();
        let key = self.key.expect("key missing");
        self.manager
            .set_listener(key, ComponentEventListener::new(kind, cb));
        self
    }

    pub fn with_listener_block(mut self, cve: ComponentEventListener<ProgramState>) -> Self {
        self.create_key_if_possible();
        let key = self.key.expect("key missing");
        self.manager.set_listener(key, cve);
        self
    }

    pub fn with_drag(self, enable: bool) -> Self {
        if enable {
            self.with_listener_block(ComponentEventListener::new(
                GuiEventKind::OnDrag,
                Box::new(|info| {
                    if let EventKind::MouseMove { dx, dy, .. } = info.event {
                        let disp = Vec2::from([dx as f32, dy as f32]);
                        info.gui_comp_tree
                            .get_mut(info.key)
                            .expect("invalid key")
                            .translate(disp);
                    }
                    None
                }),
            ))
        } else {
            self
        }
    }

    pub fn build(mut self) -> GuiComponentKey {
        self.create_key_if_possible();
        self.key.expect("builder incomplete")
    }
}
