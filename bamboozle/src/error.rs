use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouteError {
    #[error("Route already exists: {0}")]
    AlreadyExists(String),
    #[error("Route not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Route(#[from] RouteError),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Route(RouteError::AlreadyExists(_)) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            AppError::Route(RouteError::NotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(e) => {
                tracing::error!(error = %e, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
