use super::app::AppError;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error(transparent)]
    HoundError(#[from] hound::Error),
}

impl From<AnalyzerError> for AppError {
    fn from(error: AnalyzerError) -> Self {
        match error {
            AnalyzerError::HoundError(e) => AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("HoundError: {e}"),
            },
        }
    }
}
