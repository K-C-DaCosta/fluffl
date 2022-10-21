use super::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{BinaryType, CloseEvent, Event, MessageEvent, WebSocket};

pub struct WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    State: 'static,
{
    client_state: Rc<RefCell<(bool, State)>>,
    on_message_cb: Option<Closure<dyn FnMut(MessageEvent)>>,
    on_close_cb: Option<Closure<dyn FnMut(CloseEvent)>>,
    on_error_cb: Option<Closure<dyn FnMut(Event)>>,
    socket: Option<WebSocket>,
    phantom: PhantomData<(MessageCB, CloseCB, ErrorCB)>,
}

impl<State, MessageCB, CloseCB, ErrorCB> WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    CloseCB: Fn(&mut State) + Copy + 'static,
    ErrorCB: Fn(&mut State) + Copy + 'static,
    MessageCB: Fn(&mut dyn HasWebSocketClient, &mut State, &[u8]) + Copy + 'static,
    State: 'static,
{
    pub fn new(client_state: State) -> NetIncomplete<Self> {
        NetIncomplete::new(Self {
            client_state: Rc::new(RefCell::new((true, client_state))),
            on_message_cb: None,
            on_close_cb: None,
            on_error_cb: None,
            socket: None,
            phantom: PhantomData::default(),
        })
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> WebSocketBuilder<MessageCB, CloseCB, ErrorCB>
    for NetIncomplete<WsClient<State, MessageCB, CloseCB, ErrorCB>>
where
    CloseCB: Fn(&mut State) + Copy + 'static,
    MessageCB: Fn(&mut dyn HasWebSocketClient, &mut State, &[u8]) + Copy + 'static,
    ErrorCB: Fn(&mut State) + Copy + 'static,
    State: 'static,
{
    type InnerType = WsClient<State, MessageCB, CloseCB, ErrorCB>;

    fn with_on_close_cb(mut self, callback: CloseCB) -> Self {
        let state_clone = self.inner.client_state.clone();

        let js_callback = Closure::wrap(Box::new(move |_event: CloseEvent| {
            let (is_closed, state) = &mut *state_clone.borrow_mut();
            *is_closed = true;
            callback(state);
        }) as Box<dyn FnMut(CloseEvent)>);

        self.inner.on_close_cb = Some(js_callback);
        self
    }

    fn with_on_error_cb(mut self, callback: ErrorCB) -> Self {
        let state_clone = self.inner.client_state.clone();

        let js_callback = Closure::wrap(Box::new(move |_event: Event| {
            let (_is_closed, state) = &mut *state_clone.borrow_mut();
            callback(state);
        }) as Box<dyn FnMut(Event)>);

        self.inner.on_error_cb = Some(js_callback);

        self
    }

    fn with_on_message_cb(mut self, callback: MessageCB) -> Self {
        let socket_state = self.inner.client_state.clone();

        // Moving a raw pointer into the closure works so far
        // But I could see browser panics/segfaults later down the road
        // I'm definitely not comfortable with this, and honestly, im not sure why this even works at all.
        let innner_pointer = (&mut self.inner) as *mut dyn HasWebSocketClient;

        let js_callback: Closure<dyn FnMut(MessageEvent)> =
            Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(data) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let byte_array: Vec<u8> = js_sys::Uint8Array::new(&data).to_vec();
                    let (_, state) = &mut *socket_state.borrow_mut();
                    callback(unsafe { &mut *innner_pointer }, state, &byte_array);
                }
            }) as Box<dyn FnMut(MessageEvent)>);

        self.inner.on_message_cb = Some(js_callback);
        self
    }
    fn connect(mut self, uri: &str) -> Result<Self::InnerType, WebsocketError> {
        WebSocket::new(uri)
            .map(move |socket| {
                //set onclose here
                if let Some(js_callback) = self.inner.on_close_cb.as_ref() {
                    let js_function: &js_sys::Function = js_callback.as_ref().unchecked_ref();
                    socket.set_onclose(Some(js_function));
                }

                //set onerror here
                if let Some(js_callback) = self.inner.on_error_cb.as_ref() {
                    let js_function: &js_sys::Function = js_callback.as_ref().unchecked_ref();
                    socket.set_onerror(Some(js_function));
                }

                //set onmessage cb
                if let Some(js_callback) = self.inner.on_message_cb.as_ref() {
                    let js_function: &js_sys::Function = js_callback.as_ref().unchecked_ref();
                    socket.set_onmessage(Some(js_function));
                }

                // According to the wasm-bindgen example 'ArrayBuffer' is better for small messages
                // so I've decided ArrayBuffer is probably the right way to go for this library
                socket.set_binary_type(BinaryType::Arraybuffer);

                self.inner.socket = Some(socket);
                self.inner.client_state.borrow_mut().0 = false;
                self.inner
            })
            .map_err(|_err| WebsocketError::ConnectFailed {
                details: String::from("not implemented ಠ_ಠ"),
            })
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> HasWebSocketClient
    for WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    State: 'static,
{
    fn send(&mut self, data: &[u8]) -> Result<(), WebsocketError> {
        self.socket
            .as_ref()
            .map(|socket| socket.send_with_u8_array(data))
            .unwrap_or_else(|| Ok(()))
            .map_err(|_err| WebsocketError::SendFailed {
                details: String::new(),
            })
    }

    fn is_closed(&self) -> bool {
        self.client_state.borrow().0
    }

    fn listen(&mut self) {
        //the javascript runtime does this for us
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> Drop for WsClient<State, MessageCB, CloseCB, ErrorCB> {
    fn drop(&mut self) {}
}

impl<State, MessageCB, CloseCB, ErrorCB> From<WsClient<State, MessageCB, CloseCB, ErrorCB>>
    for Box<dyn HasWebSocketClient>
where
    State: 'static,
    MessageCB: 'static,
    CloseCB: 'static,
    ErrorCB: 'static,
{
    fn from(a: WsClient<State, MessageCB, CloseCB, ErrorCB>) -> Self {
        Box::new(a)
    }
}
