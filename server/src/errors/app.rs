use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseError {
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(json!(ResponseError {
                message: self.message,
            })),
        )
            .into_response()
    }
}
