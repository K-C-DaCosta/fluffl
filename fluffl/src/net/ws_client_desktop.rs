use super::{HasWebSocketClient, *};

use native_tls::TlsStream;
use tungstenite::{
    protocol::frame::{coding::CloseCode, CloseFrame},
    stream::Stream,
    Error, Message, WebSocket,
};

use std::borrow::Cow;
use std::net::TcpStream;

pub struct WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    MessageCB: Fn(  &mut dyn HasWebSocketClient,  &mut State, &[u8]) + Copy,
    CloseCB: Fn(&mut State) + Copy,
    ErrorCB: Fn(&mut State) + Copy,
{
    client_state: State,
    on_message_cb: Option<MessageCB>,
    on_close_cb: Option<CloseCB>,
    on_error_cb: Option<ErrorCB>,
    socket: Option<WebSocket<Stream<TcpStream, TlsStream<TcpStream>>>>,
    request: Option<http::Response<()>>,
    is_closed: bool,
}

impl<State, MessageCB, CloseCB, ErrorCB> WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    ErrorCB: Fn(&mut State) + Copy,
    CloseCB: Fn(&mut State) + Copy,
    MessageCB: Fn(  &mut dyn HasWebSocketClient,  &mut State, &[u8]) + Copy,
{
    pub fn new(client_state: State) -> NetIncomplete<Self> {
        NetIncomplete::new(Self {
            client_state,
            on_message_cb: None,
            on_close_cb: None,
            on_error_cb: None,
            socket: None,
            request: None,
            is_closed: true,
        })
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> WebSocketBuilder<MessageCB, CloseCB, ErrorCB>
    for NetIncomplete<WsClient<State, MessageCB, CloseCB, ErrorCB>>
where
    CloseCB: Fn(&mut State) + Copy,
    ErrorCB: Fn(&mut State) + Copy,
    MessageCB: Fn(  &mut dyn HasWebSocketClient,  &mut State, &[u8]) + Copy,
{
    type InnerType = WsClient<State, MessageCB, CloseCB, ErrorCB>;
    fn connect(mut self, uri: &str) -> Result<Self::InnerType, WebsocketError> {
        tungstenite::connect(uri)
            .map(|(socket, request)| {
                println!("partially connected(so far so good)");
                if let Stream::Plain(socket) = socket.get_ref() {
                    socket
                        .set_nonblocking(true)
                        .expect("attempt to set non-blocking failed");
                }
                self.inner.socket = Some(socket);
                self.inner.request = Some(request);
                self.inner.is_closed = false;
                self.inner
            })
            .map_err(|err| {
                println!("{}", err.to_string());
                WebsocketError::ConnectFailed {
                    details: err.to_string(),
                }
            })
    }

    fn with_on_message_cb(mut self, callback: MessageCB) -> Self {
        self.inner.on_message_cb = Some(callback);
        self
    }

    fn with_on_error_cb(mut self, callback: ErrorCB) -> Self {
        self.inner.on_error_cb = Some(callback);
        self
    }

    fn with_on_close_cb(mut self, callback: CloseCB) -> Self {
        self.inner.on_close_cb = Some(callback);
        self
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> HasWebSocketClient
    for WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    CloseCB: Fn(&mut State) + Copy,
    ErrorCB: Fn(&mut State) + Copy,
    MessageCB: Fn(  &mut dyn HasWebSocketClient,  &mut State, &[u8]) + Copy,
{
    fn send(&mut self, data: &[u8]) -> Result<(), WebsocketError> {
        if let Some(socket) = &mut self.socket {
            let buffer: Vec<u8> = data.iter().map(|&a| a).collect();
            socket
                .write_message(Message::Binary(buffer))
                .map_err(|err| WebsocketError::SendFailed {
                    details: err.to_string(),
                })
        } else {
            Ok(())
        }
    }

    fn is_closed(&self) -> bool {
        self.is_closed
    }

    fn listen(&mut self) {
        if self.socket.as_ref().unwrap().can_read() {
            let data_opt = self
                .socket
                .as_mut()
                .map(|socket| socket.read_message())
                .map(|read_res| read_res.map(|message| message.into_data()));

            match data_opt {
                Some(Ok(data)) => {                    
                    let pointer = self as *mut dyn HasWebSocketClient;
                    let force_split_borrow =  unsafe{&mut *pointer};
                    let on_message = self.on_message_cb.unwrap();
                    let client_state = &mut self.client_state; 
                    on_message(force_split_borrow,client_state, &data[..]);
                }
                Some(Err(err)) => match err {
                    Error::ConnectionClosed => {
                        let on_close = self.on_close_cb.unwrap();
                        on_close(&mut self.client_state);
                        self.is_closed = true
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> Drop for WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    CloseCB: Fn(&mut State) + Copy,
    ErrorCB: Fn(&mut State) + Copy,
    MessageCB: Fn(  &mut dyn HasWebSocketClient,  &mut State, &[u8]) + Copy,
{
    fn drop(&mut self) {
        let code = CloseFrame {
            code: CloseCode::Normal,
            reason: Cow::from("client dropped socket"),
        };

        let on_close = self.on_close_cb.unwrap();
        on_close(&mut self.client_state);
        self.socket.as_mut().map(|socket| socket.close(Some(code)));
    }
}

impl<State, MessageCB, CloseCB, ErrorCB> Into<Box<dyn HasWebSocketClient>>
    for WsClient<State, MessageCB, CloseCB, ErrorCB>
where
    MessageCB: Fn(  &mut dyn HasWebSocketClient,  &mut State, &[u8]) + Copy + 'static,
    CloseCB: Fn(&mut State) + Copy + 'static,
    ErrorCB: Fn(&mut State) + Copy + 'static,
    State: 'static,
{
    fn into(self) -> Box<dyn HasWebSocketClient> {
        Box::new(self)
    }
}
