// client websocket
pub type MutexWebSocketClientWriter = std::sync::Arc<
    tokio::sync::Mutex<
        futures_util::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>,
    >,
>;
pub type WebSocketClientReader = futures_util::stream::SplitStream<axum::extract::ws::WebSocket>;

// server websocket
pub type WebSocketServerWriter = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Message,
>;
pub type WebSocketServerReader = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;
