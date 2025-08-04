use crate::models::packet::WindowPacket;

use super::app::AppError;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("UnexpectedMessageTypeError: unsupported message type received")]
    UnexpectedMessageTypeError,
    #[error("UnexpectedMessageError: {0}")]
    UnexpectedMessageError(String),
    #[error(transparent)]
    SetGlobalDefaultError(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
    #[error(transparent)]
    TokioTungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("AudioInfoError: {0}")]
    AudioInfoError(String),
    #[error(transparent)]
    MpscVecU8SenderError(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),
    #[error(transparent)]
    MpscWindowPacketSenderError(#[from] tokio::sync::mpsc::error::SendError<WindowPacket>),
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    RmpSerdeEncodeError(#[from] rmp_serde::encode::Error),
    #[error("ParseAudioInfoError: Invalid audio info format: {0}")]
    ParseAudioInfoError(String),
    #[error("AudioInfoUndefinedError: Audio info is not set")]
    AudioInfoUndefinedError,
    #[error(transparent)]
    PyError(#[from] pyo3::PyErr),
}

impl From<HandlerError> for AppError {
    fn from(error: HandlerError) -> Self {
        match error {
            HandlerError::UnexpectedMessageTypeError => AppError {
                status_code: StatusCode::BAD_REQUEST,
                message: "UnexpectedMessageTypeError: unsupported message type received".into(),
            },
            HandlerError::UnexpectedMessageError(e) => AppError {
                status_code: StatusCode::BAD_REQUEST,
                message: format!("UnexpectedMessageError: {e}"),
            },
            HandlerError::SetGlobalDefaultError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("SetGlobalDefaultError: {e}"),
            },
            HandlerError::IoError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("IoError: {e}"),
            },
            HandlerError::AxumError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("AxumError: {e}"),
            },
            HandlerError::TokioTungsteniteError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("TokioTungsteniteError: {e}"),
            },
            HandlerError::ParseIntError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("ParseIntError: {e}"),
            },
            HandlerError::AudioInfoError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("AudioInfoError: {e}"),
            },
            HandlerError::MpscVecU8SenderError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("MpscVecU8SenderError: {e}"),
            },
            HandlerError::MpscWindowPacketSenderError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("MpscWindowPacketSenderError: {e}"),
            },
            HandlerError::TokioJoinError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("TokioJoinError: {e}"),
            },
            HandlerError::RmpSerdeEncodeError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("RmpSerdeEncodeError: {e}"),
            },
            HandlerError::ParseAudioInfoError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("ParseAudioInfoError: Invalid audio info format: {e}"),
            },
            HandlerError::AudioInfoUndefinedError => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "AudioInfoUndefinedError: Audio info is not set".into(),
            },
            HandlerError::PyError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("PyError: {e}"),
            },
        }
    }
}
