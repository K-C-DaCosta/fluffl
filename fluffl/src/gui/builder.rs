use super::*;

pub trait HasComponentBuilder<ProgramState>: Sized {
    type ComponentKind: GuiComponent + 'static;
    fn manager(&mut self) -> &mut GuiManager<ProgramState>;
    fn parent(&mut self) -> &mut Option<GuiComponentKey>;
    fn key(&mut self) -> &mut Option<GuiComponentKey>;
    fn state(&mut self) -> &mut Option<Self::ComponentKind>;
    fn build(self) -> GuiComponentKey;

    fn with_name<T: AsRef<str>>(mut self, name: T) -> Self {
        self.state()
            .as_mut()
            .expect("state not initalized")
            .set_name(name.as_ref());
        self
    }

    fn with_parent(mut self, parent: GuiComponentKey) -> Self {
        *self.parent() = Some(parent);
        self
    }

    fn with_listener<Listener>(self, kind: GuiEventKind, mut listener: Listener) -> Self
    where
        Listener: FnMut(&mut Self::ComponentKind, EventKind, &mut MutationRequestQueue<ProgramState>)
            + 'static,
    {
        self.with_listener_advanced(
            kind,
            Box::new(move |event_state| {
                let event = event_state.event;
                let key = event_state.key;
                let mutation_queue = event_state.mutation_queue;

                let comp = event_state
                    .gui_comp_tree
                    .get_mut(key)?
                    .as_any_mut()
                    .downcast_mut::<Self::ComponentKind>()?;
                listener(comp, event, mutation_queue);
                None
            }),
        )
    }

    fn with_listener_advanced(
        mut self,
        kind: GuiEventKind,
        cb: ListenerCallBack<ProgramState>,
    ) -> Self {
        let key = self.key().expect("key missing");
        self.manager()
            .push_listener(key, ComponentEventListener::new(kind, cb));
        self
    }

    fn with_listener_block(mut self, cve: ComponentEventListener<ProgramState>) -> Self {
        let key = self.key().expect("key missing");
        self.manager().push_listener(key, cve);
        self
    }

    fn with_drag(self, enable: bool) -> Self {
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

    /// drags the highest ancestor that ISN'T the origin
    fn with_drag_highest(self, enable: bool) -> Self {
        if enable {
            self.with_listener_advanced(
                GuiEventKind::OnDrag,
                Box::new(|info| {
                    let mut cur_node_key = info.key;
                    let comp_tree = info.gui_comp_tree;
                    let disp = info.event.disp();

                    //walk up the tree to find the root component
                    while let Some(parent) = comp_tree.get_parent_id(cur_node_key) {
                        if comp_tree.get(parent).unwrap().is_origin() {
                            break;
                        }
                        cur_node_key = parent.into();
                    }

                    let root_bounds = {
                        let root = comp_tree.get_mut(cur_node_key).expect("root not found");
                        root.translate(disp);
                        root.bounds()
                    };

                    let cur_node = comp_tree
                        .get_mut(info.key)
                        .expect("gui manager somehow sent invalid key");
                    let cur_bounds = cur_node.bounds();

                    cur_node.set_bounds(Vec2::from([root_bounds[0], cur_bounds[1]]));

                    None
                }),
            )
        } else {
            self
        }
    }
}
