use crate::{
    errors::handler::HandlerError,
    models::ws::{WebSocketClientReader, WebSocketServerWriter},
};
use axum::extract::ws::Message;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite;

// [task1] client -> server
pub async fn handle_client_to_server(
    mut client_reader: WebSocketClientReader,
    mut server_writer: WebSocketServerWriter,
) -> Result<(), HandlerError> {
    while let Some(Ok(message)) = client_reader.next().await {
        match message {
            Message::Text(text) => {
                tracing::info!("Received text from client: {:?}", text);
                // send only "open" or "accept" messages to server
                //* step1: receive open message from client and send to server *//
                //* step4: receive accept message from client and send to server *//
                if text == "open" || text == "accept" {
                    tracing::info!("Forwarding message from client to server: {}", text);
                    server_writer
                        .send(tungstenite::Message::Text(text.to_string().into()))
                        .await
                        .map_err(HandlerError::TokioTungsteniteError)?;
                }
            }
            Message::Close(close) => {
                tracing::info!("Client disconnected: {:?}", close);
                //? send close frame to server //
                // convert axum CloseFrame to tungstenite CloseFrame
                let tungstenite_close =
                    close.map(
                        |close_frame| tokio_tungstenite::tungstenite::protocol::CloseFrame {
                            code: tungstenite::protocol::frame::coding::CloseCode::from(
                                close_frame.code,
                            ),
                            reason: close_frame.reason.to_string().into(),
                        },
                    );
                // send close frame to server
                server_writer
                    .send(tungstenite::Message::Close(tungstenite_close))
                    .await
                    .map_err(HandlerError::TokioTungsteniteError)?;
            }
            _ => {
                tracing::error!("Received unsupported message type from client");
                return Err(HandlerError::UnexpectedMessageTypeError);
            }
        };
    }
    Ok(())
}
