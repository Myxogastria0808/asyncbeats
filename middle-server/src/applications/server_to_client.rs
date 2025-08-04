use crate::{
    errors::handler::HandlerError,
    models::{
        audio::{AudioInfo, RwLockAudioInfo},
        ws::{MutexWebSocketClientWriter, WebSocketServerReader},
    },
};
use axum::extract::ws::Message;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;

// [task2] server -> client
pub async fn handle_server_to_client(
    mut server_reader: WebSocketServerReader,
    pcm_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    shared_client_writer: MutexWebSocketClientWriter,
    shared_audio_info: RwLockAudioInfo,
) -> Result<(), HandlerError> {
    while let Some(Ok(message)) = server_reader.next().await {
        match message {
            tungstenite::Message::Text(text) => {
                //* step2: receive audio info from server *//
                tracing::info!("Received text from client: {:?}", text);
                // set audio info
                let mut shared_audio_info = shared_audio_info.write().await;
                *shared_audio_info = AudioInfo::try_from(text.clone()).map_err(|e| {
                    tracing::error!("Failed to parse audio info: {:?}", e);
                    HandlerError::ParseAudioInfoError(e.to_string())
                })?;
                drop(shared_audio_info); // release the lock
                //* step3: send audio info to client *//
                let mut writer = shared_client_writer.lock().await;
                writer
                    .send(Message::Text(text.to_string().into()))
                    .await
                    .map_err(HandlerError::AxumError)?;
            }
            tungstenite::Message::Binary(binary) => {
                //* step5: receive PCM data from server *//
                tracing::info!("Received binary from client: {:?}", binary);
                //? Sender (Producer) //
                pcm_tx.send(binary.to_vec()).await?;
            }
            tungstenite::Message::Close(close) => {
                tracing::info!("Server disconnected: {:?}", close);
                let mut writer = shared_client_writer.lock().await;
                //? send close frame to client //
                // convert tungstenite CloseFrame to axum CloseFrame
                let axum_close = close.map(|close_frame| axum::extract::ws::CloseFrame {
                    code: axum::extract::ws::CloseCode::from(u16::from(close_frame.code)),
                    reason: close_frame.reason.to_string().into(),
                });
                // send close frame to client
                writer
                    .send(axum::extract::ws::Message::Close(axum_close))
                    .await
                    .map_err(HandlerError::AxumError)?;
            }
            _ => {
                tracing::error!("Received unsupported message type from server");
                return Err(HandlerError::UnexpectedMessageTypeError);
            }
        }
    }
    Ok(())
}
