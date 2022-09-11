use super::*;

const MAX_EVENT_LISTENERS: usize = 16;

/// User defined handlers for all events
pub struct ComponentHandlerBlock<ProgramState> {
    handlers: Vec<Vec<ListenerCallBack<ProgramState>>>,
}

impl<ProgramState> ComponentHandlerBlock<ProgramState> {
    pub fn new() -> Self {
        let mut handlers: Vec<Vec<ListenerCallBack<ProgramState>>> = vec![];
        for _ in 0..MAX_EVENT_LISTENERS {
            handlers.push(Vec::new());
        }
        Self { handlers }
    }

    pub fn clear_handlers(&mut self, kind: GuiEventKind) {
        self.handlers[kind as usize].clear();
    }

    pub fn push_handler(&mut self, listener: ComponentEventListener<ProgramState>) {
        self.handlers[listener.kind as usize].push(listener.callback);
    }

    pub fn fire_handler<'a>(
        &mut self,
        kind: GuiEventKind,
        state: EventListenerInfo<'a, ProgramState>,
    ) {
        for handle in self.handlers[kind as usize].iter_mut() {
            let state: EventListenerInfo<ProgramState> =
                unsafe { std::mem::transmute_copy(&state) };
            handle(state);
        }
    }
}

impl<ProgramState> std::ops::Deref for ComponentHandlerBlock<ProgramState> {
    type Target = Vec<Vec<ListenerCallBack<ProgramState>>>;
    fn deref(&self) -> &Self::Target {
        &self.handlers
    }
}