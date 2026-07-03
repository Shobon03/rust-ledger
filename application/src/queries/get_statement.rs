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
  async fn get_statement(&self, account_id: &AccountId) -> Result<AccountStatement, AppError>;
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
    fields(account_id = %account_id)
  )]
  pub async fn execute(&self, account_id: AccountId) -> Result<AccountStatement, AppError> {
    info!("Querying account statement");
    match self.repository.get_statement(&account_id).await {
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
