use super::app::AppError;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum StreamerError {
    #[error(transparent)]
    HoundError(#[from] hound::Error),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
}

impl From<StreamerError> for AppError {
    fn from(error: StreamerError) -> Self {
        match error {
            StreamerError::HoundError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("HoundError: {e}"),
            },
            StreamerError::AxumError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("AxumError: {e}"),
            },
        }
    }
}
