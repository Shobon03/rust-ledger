use crate::value_objects::account::AccountId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
  #[error("Tranasaction is unbalanced: debits and credits do not match")]
  UnbalancedTransaction,

  #[error("Amount must be greater than zero")]
  ZeroAmount,

  #[error("Insufficient funds for account: {0}")]
  InsufficientFunds(AccountId),
}
