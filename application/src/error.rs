use domain::{error::DomainError, value_objects::transaction::TransactionId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("Domain rule violation: {0}")]
  Domain(#[from] DomainError),

  #[error("Infrastructure error: {0}")]
  Infrastructure(String),

  #[error("Idempotency key was already processed. Returning original transaction")]
  IdempotencyConflict(TransactionId),
}
