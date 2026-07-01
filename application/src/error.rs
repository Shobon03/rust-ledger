use domain::error::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("Domain rule violation: {0}")]
  Domain(#[from] DomainError),

  #[error("Infrastructure error: {0}")]
  Infrastructure(String),
}
