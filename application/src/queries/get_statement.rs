use crate::error::AppError;
use domain::{entities::transaction::Transaction, value_objects::account::AccountId};
use std::sync::Arc;

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

  pub async fn execute(&self, account_id: AccountId) -> Result<AccountStatement, AppError> {
    self.repository.get_statement(&account_id).await
  }
}
