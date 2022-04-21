#[cfg(not(all(target_family = "wasm", not(target_os = "wasi"))))]
#[path = "./net/ws_client_desktop.rs"]
pub mod ws_client;

#[cfg(all(target_family = "wasm", not(target_os = "wasi")))]
#[path = "./net/ws_client_web.rs"]
pub mod ws_client;

pub use ws_client::*;

#[derive(Clone)]
pub enum WebsocketError {
    ConnectFailed { details: String },
    SendFailed { details: String },
}

pub struct NetIncomplete<T> {
    inner: T,
}

impl<T> NetIncomplete<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

pub trait WebSocketBuilder<MessageCallback, CloseCallback,ErrorCallback>
{
    type InnerType;
    fn with_on_message_cb(self, callback: MessageCallback) -> Self;
    fn with_on_close_cb(self, callback: CloseCallback) -> Self;
    fn with_on_error_cb(self, callback: ErrorCallback) -> Self;
    fn connect(self, uri: &str) -> Result<Self::InnerType, WebsocketError>;
}

pub trait HasWebSocketClient
{
    fn send(&mut self, data: &[u8]) -> Result<(), WebsocketError>;
    fn is_closed(&self) -> bool;
    fn listen(&mut self);

}
