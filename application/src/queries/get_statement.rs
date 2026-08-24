use crate::error::AppError;
use domain::{entities::transaction::Transaction, value_objects::account::AccountId};
use std::sync::Arc;
use tracing::{error, info, instrument};

pub struct AccountStatement {
  pub account_id: AccountId,
  pub transactions: Vec<Transaction>,
}

#[async_trait::async_trait]
pub trait StatementQueryRepository: Send + Sync {
  async fn get_statement(
    &self,
    account_id: &AccountId,
    limit: usize,
    offset: usize,
  ) -> Result<AccountStatement, AppError>;
}

pub struct GetStatementQuery {
  repository: Arc<dyn StatementQueryRepository + Send + Sync>,
}

impl GetStatementQuery {
  pub fn new(repository: Arc<dyn StatementQueryRepository>) -> Self {
    Self { repository }
  }

  #[instrument(
    skip(self),
    fields(account_id = %account_id, limit = ?limit, offset = ?offset)
  )]
  pub async fn execute(
    &self,
    account_id: AccountId,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> Result<AccountStatement, AppError> {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);

    info!("Querying account statement");
    match self.repository.get_statement(&account_id, limit, offset).await {
      Ok(statement) => {
        info!(transactions_count = statement.transactions.len(), "Statement query succeeded");
        Ok(statement)
      }
      Err(e) => {
        error!(error = %e, "Statement query failed");
        Err(e)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use domain::{entities::entry::{EntryLine, EntryType}, value_objects::idempotency::IdempotencyKeyId};
  use uuid::Uuid;

  struct MockStatementQueryRepo {
    pub transactions: Vec<Transaction>,
  }

  #[async_trait::async_trait]
  impl StatementQueryRepository for MockStatementQueryRepo {
    async fn get_statement(
      &self,
      account_id: &AccountId,
      limit: usize,
      offset: usize,
    ) -> Result<AccountStatement, AppError> {
      let transactions = self.transactions
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();

      Ok(AccountStatement {
        account_id: *account_id,
        transactions,
      })
    }
  }

  #[tokio::test]
  async fn get_statement_returns_transactions() {
    let account_id = AccountId(Uuid::now_v7());
    let other_account = AccountId(Uuid::now_v7());
    let id_key = IdempotencyKeyId(Uuid::now_v7());

    let debit = EntryLine {
      account_id,
      amount: 100,
      operation: EntryType::Debit,
    };
    let credit = EntryLine {
      account_id: other_account,
      amount: 100,
      operation: EntryType::Credit,
    };
    let transaction = Transaction::new(id_key, vec![debit, credit]).unwrap();

    let repo = Arc::new(MockStatementQueryRepo {
      transactions: vec![transaction.clone()],
    });
    let query = GetStatementQuery::new(repo);

    let result = query.execute(account_id, Some(10), Some(0)).await;

    assert!(result.is_ok());
    let statement = result.unwrap();
    assert_eq!(statement.account_id, account_id);
    assert_eq!(statement.transactions.len(), 1);
    assert_eq!(statement.transactions[0].id.0, transaction.id.0);
  }
}
