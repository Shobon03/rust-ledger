use crate::error::AppError;
use domain::value_objects::account::AccountId;
use std::sync::Arc;

pub struct AccountBalance {
  pub account_id: AccountId,
  pub balance: i64,
}

#[async_trait::async_trait]
pub trait BalanceQueryRepository: Send + Sync {
  async fn get_balance(&self, account_id: &AccountId) -> Result<AccountBalance, AppError>;
}

pub struct GetBalanceQuery {
  repository: Arc<dyn BalanceQueryRepository + Send + Sync>,
}

impl GetBalanceQuery {
  pub fn new(repository: Arc<dyn BalanceQueryRepository>) -> Self {
    Self { repository }
  }

  pub async fn execute(&self, account_id: AccountId) -> Result<AccountBalance, AppError> {
    self.repository.get_balance(&account_id).await
  }
}
