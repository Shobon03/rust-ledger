use crate::{
  entities::entry::{EntryLine, EntryType},
  error::DomainError,
  value_objects::{idempotency::IdempotencyKeyId, transaction::TransactionId},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::value_objects::account::AccountId;

  fn setup_mockup_accounts() -> (AccountId, AccountId, IdempotencyKeyId) {
    (
      AccountId(Uuid::now_v7()),
      AccountId(Uuid::now_v7()),
      IdempotencyKeyId(Uuid::now_v7()),
    )
  }

  #[test]
  fn transaction_must_be_balanced_to_succeed() {
    let (from, to, idempotency_key) = setup_mockup_accounts();

    let debit = EntryLine {
      account_id: from,
      amount: 1000,
      operation: EntryType::Debit,
    };

    let credit = EntryLine {
      account_id: to,
      amount: 1000,
      operation: EntryType::Credit,
    };

    let transaction = Transaction::new(idempotency_key, vec![debit, credit]);
    assert!(transaction.is_ok(), "Balanced transaction must be accepted");
  }

  #[test]
  fn transaction_fails_when_unbalanced() {
    let (from, to, idempotency_key) = setup_mockup_accounts();

    let debit = EntryLine {
      account_id: from,
      amount: 1000,
      operation: EntryType::Debit,
    };

    let credit = EntryLine {
      account_id: to,
      amount: 500,
      operation: EntryType::Credit,
    };

    let transaction = Transaction::new(idempotency_key, vec![debit, credit]);
    assert!(transaction.is_err());
    match transaction.unwrap_err() {
      DomainError::UnbalancedTransaction => {}
      _ => panic!("Returned incorrect domain error"),
    }
  }
}
