use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    InvalidJwt(String),
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
    #[error(transparent)]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    ApiPathRejection(#[from] axum::extract::rejection::PathRejection),
    #[error("{0}")]
    UnknownApiVerRejection(String),
    #[error("{0}")]
    Error(String),
}
