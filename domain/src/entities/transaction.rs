use crate::{
  entities::entry::{EntryLine, EntryType},
  error::DomainError,
  value_objects::{idempotency::IdempotencyKeyId, transaction::TransactionId},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
  pub id: TransactionId,
  pub idempotency_key: IdempotencyKeyId,
  pub timestamp: u64,
  pub lines: Vec<EntryLine>,
}

impl Transaction {
  pub fn new(
    idempotency_key: IdempotencyKeyId,
    lines: Vec<EntryLine>,
  ) -> Result<Self, DomainError> {
    let mut debits = 0;
    let mut credits = 0;

    for line in &lines {
      match line.operation {
        EntryType::Debit => debits += line.amount,
        EntryType::Credit => credits += line.amount,
      }
    }

    if debits != credits {
      return Err(DomainError::UnbalancedTransaction);
    }

    Ok(Self {
      id: TransactionId(Uuid::now_v7()),
      idempotency_key,
      timestamp: chrono::Utc::now().timestamp() as u64,
      lines,
    })
  }
}
