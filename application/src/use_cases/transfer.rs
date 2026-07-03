use crate::{error::AppError, ports::repository::LedgerRepository};
use domain::{
  entities::{
    entry::{EntryLine, EntryType},
    transaction::Transaction,
  },
  error::DomainError,
  value_objects::{account::AccountId, idempotency::IdempotencyKeyId, transaction::TransactionId},
};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

pub struct TransferRequest {
  pub idempotency_key: IdempotencyKeyId,
  pub from_account: AccountId,
  pub to_account: AccountId,
  pub amount: u64,
}

pub struct TransferUseCase {
  repository: Arc<dyn LedgerRepository + Send + Sync>,
}

impl TransferUseCase {
  pub fn new(repository: Arc<dyn LedgerRepository>) -> Self {
    Self { repository }
  }

  #[instrument(
    skip(self, req),
    fields(
      from = %req.from_account,
      to= %req.to_account,
      amount = req.amount
    )
  )]
  pub async fn execute(&self, req: TransferRequest) -> Result<TransactionId, AppError> {
    info!("Processing transfer request");

    if req.amount == 0 {
      warn!("Transfer rejected: amount must be greater than zero");
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
    match self.repository.save_transaction(&transaction).await {
      Ok(tansaction_id) => {
        info!("Transfer completed successfully");
        Ok(tansaction_id)
      }
      Err(AppError::IdempotencyConflict(existing_id)) => {
        info!("Transaction already processed, returning cached transaction ID");
        Ok(existing_id)
      }
      Err(e) => {
        error!(error = %e, "Failed to persist transfer transaction");
        Err(e)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::Mutex;
  use uuid::Uuid;

  struct MockLedgerRepo {
    pub saved_transactions: Mutex<Vec<Transaction>>,
  }

  impl MockLedgerRepo {
    fn new() -> Self {
      Self {
        saved_transactions: Mutex::new(Vec::new()),
      }
    }
  }

  #[async_trait::async_trait]
  impl LedgerRepository for MockLedgerRepo {
    async fn save_transaction(&self, transaction: &Transaction) -> Result<TransactionId, AppError> {
      let mut lock = self.saved_transactions.lock().unwrap();
      lock.push(transaction.clone());

      Ok(transaction.id)
    }
  }

  #[tokio::test]
  async fn transfer_executes_and_calls_repository() {
    let repo = Arc::new(MockLedgerRepo::new());
    let use_case = TransferUseCase::new(repo.clone());

    let req = TransferRequest {
      idempotency_key: IdempotencyKeyId(Uuid::now_v7()),
      from_account: AccountId(Uuid::now_v7()),
      to_account: AccountId(Uuid::now_v7()),
      amount: 5000,
    };

    let result = use_case.execute(req).await;

    assert!(result.is_ok(), "Use case must return success");

    let saved = repo.saved_transactions.lock().unwrap();
    assert_eq!(saved.len(), 1, "Repo must have been called exaclty once");
  }

  #[tokio::test]
  async fn transfer_fails_with_zero_amount() {
    let repo = Arc::new(MockLedgerRepo::new());
    let use_case = TransferUseCase::new(repo.clone());

    let req = TransferRequest {
      idempotency_key: IdempotencyKeyId(Uuid::now_v7()),
      from_account: AccountId(Uuid::now_v7()),
      to_account: AccountId(Uuid::now_v7()),
      amount: 0,
    };

    let result = use_case.execute(req).await;

    assert!(
      result.is_err(),
      "Use case must fail when trying to transfer zero"
    );

    let saved = repo.saved_transactions.lock().unwrap();
    assert_eq!(
      saved.len(),
      0,
      "Repo must not have saved in case of validation error"
    );
  }
}
