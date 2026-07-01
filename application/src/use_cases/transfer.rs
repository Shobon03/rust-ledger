use std::sync::Arc;

use domain::{
  entities::{
    entry::{EntryLine, EntryType},
    transaction::Transaction,
  },
  error::DomainError,
  value_objects::{account::AccountId, idempotency::IdempotencyKeyId, transaction::TransactionId},
};

use crate::{error::AppError, ports::repository::LedgerRepository};

pub struct TransferRequest {
  pub idempotency_key: IdempotencyKeyId,
  pub from_account: AccountId,
  pub to_account: AccountId,
  pub amount: u64,
}

pub struct TransferUseCase {
  repository: Arc<dyn LedgerRepository>,
}

impl TransferUseCase {
  pub fn new(repository: Arc<dyn LedgerRepository>) -> Self {
    Self { repository }
  }

  pub async fn execute(&self, req: TransferRequest) -> Result<TransactionId, AppError> {
    if req.amount == 0 {
      return Err(AppError::Domain(DomainError::ZeroAmount));
    }

    let debit = EntryLine {
      account_id: req.from_account,
      amount: req.amount,
      operation: EntryType::Debit,
    };

    let credit = EntryLine {
      account_id: req.to_account,
      amount: req.amount,
      operation: EntryType::Credit,
    };

    let transaction = Transaction::new(req.idempotency_key, vec![debit, credit])?;
    self.repository.save_transaction(&transaction).await?;

    Ok(transaction.id)
  }
}
