pub mod core;
pub mod logging;
pub mod security;
pub mod websocket;

#[cfg(feature = "development")]
pub mod dev;

pub type ClientConnection =
    tokio_tungstenite::WebSocketStream<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>;

pub type ServerConnection =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
