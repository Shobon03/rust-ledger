use domain::{error::DomainError, value_objects::transaction::TransactionId};
use thiserror::Error;
use axum::{response::{IntoResponse, Response}, http::StatusCode};

#[derive(Debug, Error)]
pub enum AppError {
  #[error("Domain rule violation: {0}")]
  Domain(#[from] DomainError),

  #[error("Infrastructure error: {0}")]
  Infrastructure(String),

  #[error("Idempotency key was already processed. Returning original transaction")]
  IdempotencyConflict(TransactionId),
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    match self {
      AppError::Domain(e) => {
        tracing::warn!(error = %e, "Business validation error");
        (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
      }
      AppError::Infrastructure(e) => {
        tracing::error!(error = %e, "Internal server error");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()).into_response()
      }
      AppError::IdempotencyConflict(id) => {
        tracing::warn!(transaction_id = %id, "Idempotency conflict");
        (StatusCode::CONFLICT, format!("Idempotency conflict: {}", id)).into_response()
      }
    }
  }
}
