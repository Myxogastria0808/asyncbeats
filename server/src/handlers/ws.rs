use crate::{
    application::streamer::wave_streamer,
    errors::{app::AppError, handler::HandlerError},
    models::shared_state::RwLockSharedState,
};
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};

// handler
pub async fn websocket_handler(
    State(shared_state): State<RwLockSharedState>,
    web_socket: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let shared_state = shared_state.read().await;
    let response = web_socket.on_upgrade(|socket| async move {
        if let Err(error) = websocket_processing(socket).await {
            tracing::error!("WebSocket error: {:?}", error);
        }
    });
    drop(shared_state);
    Ok(response)
}

//websocket
pub async fn websocket_processing(mut socket: WebSocket) -> Result<(), AppError> {
    while let Some(message) = socket.recv().await {
        // Receive a message from the client
        match message {
            Ok(message) => {
                match message {
                    Message::Text(text) => {
                        // receive connection request from client
                        let msg = text.to_string();
                        if msg != "open" && msg != "accept" {
                            tracing::info!("Received unexpected text: {:?}", msg);
                            return Err(HandlerError::UnexpectedMessageError(msg).into());
                        }
                        tracing::info!("Received text: {:?}", msg);

                        // step1: analyze audio file and send audio info to middle-server
                        if msg == "open" {
                            // analyze audio file
                            let audio_info = crate::application::analyzer::wave_analyzer()?;
                            // send audio info to middle-server
                            /*
                                FORMAT: <channels> <sample_rate> <bits_per_sample> <pcm_format>
                            */
                            socket
                                .send(Message::Text(
                                    format!(
                                        "{} {} {} {}",
                                        audio_info.channels,
                                        audio_info.sample_rate,
                                        audio_info.bits_per_sample,
                                        audio_info.pcm_format
                                    )
                                    .into(),
                                ))
                                .await
                                .map_err(HandlerError::AxumError)?;
                        }

                        //step2: receive connection acceptance from middle-server and send PCM data to middle-server
                        if msg == "accept" {
                            wave_streamer(&mut socket).await?;
                        }
                    }
                    Message::Close(close) => {
                        tracing::info!("Client disconnected: {:?}", close);
                        return Ok(());
                    }
                    _ => {
                        tracing::error!("Received unsupported message type from server");
                        return Err(HandlerError::UnexpectedMessageTypeError.into());
                    }
                }
            }
            Err(error) => {
                tracing::error!("Error receiving message: {}", error);
                return Err(HandlerError::from(error).into());
            }
        }
    }
    Ok(())
}
