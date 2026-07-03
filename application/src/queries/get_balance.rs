use crate::error::AppError;
use domain::value_objects::account::AccountId;
use std::sync::Arc;
use tracing::{error, info, instrument};

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

  #[instrument(
    skip(self),
    fields(account_id = %account_id)
  )]
  pub async fn execute(&self, account_id: AccountId) -> Result<AccountBalance, AppError> {
    info!("Querying account balance");
    match self.repository.get_balance(&account_id).await {
      Ok(balance) => {
        info!(balance = balance.balance, "Balance query succeeded");
        Ok(balance)
      }
      Err(e) => {
        error!(error = %e, "Balance query failed");
        Err(e)
      }
    }
  }
}
