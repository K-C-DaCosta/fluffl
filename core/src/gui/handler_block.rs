use super::*;

const MAX_EVENT_LISTENERS: usize = 16;

/// User defined handlers for all events
pub struct ComponentHandlerBlock<ProgramState> {
    handlers: Vec<ListenerCallBack<ProgramState>>,
}

impl<ProgramState> ComponentHandlerBlock<ProgramState> {
    pub fn new() -> Self {
        let mut handlers: Vec<ListenerCallBack<ProgramState>> = vec![];
        for _ in 0..MAX_EVENT_LISTENERS {
            handlers.push(Box::new(|_| None));
        }
        Self { handlers }
    }

    pub fn set_handler(&mut self, listener: ComponentEventListener<ProgramState>) {
        self.handlers[listener.kind as usize] = listener.callback;
    }

    pub fn fire_handler<'a>(
        &mut self,
        kind: GuiEventKind,
        state: EventListenerInfo<'a, ProgramState>,
    ) {
        self.handlers[kind as usize](state);
    }
}
