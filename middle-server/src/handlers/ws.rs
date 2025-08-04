use crate::{
    applications::{
        client_to_server::handle_client_to_server, pcm::pcm_data_processing,
        server_to_client::handle_server_to_client, window::window_data_processing,
    },
    errors::{app::AppError, handler::HandlerError},
    models::{
        audio::{AudioInfo, RwLockAudioInfo},
        packet::WindowPacket,
        shared_state::RwLockSharedState,
        ws::MutexWebSocketClientWriter,
    },
};
use axum::extract::ws::WebSocket;
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;

//* constant values *//
static SERVER_URL: &str = "ws://localhost:5000";
static WINDOW_SIZE: u64 = 200;
static SLIDE_SIZE: u64 = 100;
static PCM_CHANNEL_CAPACITY: u64 = 1000;
static WINDOW_CHANNEL_CAPACITY: u64 = 1000;

// handler
pub async fn websocket_handler(
    State(_shared_state): State<RwLockSharedState>, // unused shared state
    web_socket: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let response = web_socket.on_upgrade(|socket| async move {
        if let Err(error) = websocket_processing(socket).await {
            tracing::error!("WebSocket processing error: {:?}", error);
        }
        tracing::info!("WebSocket connection closed.");
    });
    Ok(response)
}

// websocket
pub async fn websocket_processing(client_socket: WebSocket) -> Result<(), AppError> {
    // connect to the server
    let (server_socket, _) = connect_async(SERVER_URL)
        .await
        .map_err(HandlerError::TokioTungsteniteError)?;
    tracing::info!("Connection to server established.");

    // split client and server sockets
    /*
        - tokio (tokio::net::TcpStream)
        https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html#method.split

        - axum (axum::extract::ws::WebSocket)
        https://docs.rs/axum/latest/axum/extract/ws/index.html#read-and-write-concurrently

        - futures-util (futures_util::stream::StreamExt)
        https://docs.rs/futures-util/0.3.31/futures_util/stream/trait.StreamExt.html#method.split

        - tokio-tungstenite (tokio_tungstenite::WebSocketStream)
        https://stackoverflow.com/questions/68217767/where-is-the-split-method-of-tokio-tungstenitewebsocketstream-implemented
        https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/struct.WebSocketStream.html#method.split
    */
    let (client_writer, client_reader) = client_socket.split();
    let (server_writer, server_reader) = server_socket.split();

    // wrap in Arc<Mutex<T>> to safely write to the client from multiple tasks
    let shared_client_writer: MutexWebSocketClientWriter = Arc::new(Mutex::new(client_writer));

    // create tokio::sync::mpsc channel for streaming PCM data
    let (pcm_tx, pcm_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(PCM_CHANNEL_CAPACITY as usize);
    let (window_tx, window_rx) =
        tokio::sync::mpsc::channel::<WindowPacket>(WINDOW_CHANNEL_CAPACITY as usize);

    // create shared state for audio info
    let shared_audio_info: RwLockAudioInfo =
        Arc::new(tokio::sync::RwLock::new(AudioInfo::default()));

    //* --- Start independent tasks --- *//
    // [task1] client -> server
    let client_read_task = tokio::spawn(handle_client_to_server(client_reader, server_writer));
    // [task2] server -> client
    let server_read_task = tokio::spawn(handle_server_to_client(
        server_reader,
        pcm_tx,
        Arc::clone(&shared_client_writer),
        Arc::clone(&shared_audio_info),
    ));
    // [task3] pcm data processing
    let pcm_processing_task = tokio::spawn(pcm_data_processing(
        WINDOW_SIZE,
        SLIDE_SIZE,
        pcm_rx,
        window_tx,
    ));
    // [task4] window data processing
    let window_processing_task = tokio::spawn(window_data_processing(
        window_rx,
        Arc::clone(&shared_client_writer),
        Arc::clone(&shared_audio_info),
    ));

    //* When one of the tasks is completed, tokio make the other tasks also complete. *//
    (tokio::select! {
        response = client_read_task => response,
        response = server_read_task => response,
        response = pcm_processing_task => response,
        response = window_processing_task => response,
    })
    .map_err(HandlerError::TokioJoinError)??;
    Ok(())
}
