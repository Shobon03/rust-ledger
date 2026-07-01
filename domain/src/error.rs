use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
  #[error("Tranasaction is unbalanced: debits and credits do not match")]
  UnbalancedTransaction,

  #[error("Amount must be greater than zero")]
  ZeroAmount,
}
