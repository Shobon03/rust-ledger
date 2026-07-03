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

#[cfg(test)]
mod tests {
  use super::*;
  use uuid::Uuid;

  struct MockBalanceQueryRepo {
    pub balance: i64,
  }

  #[async_trait::async_trait]
  impl BalanceQueryRepository for MockBalanceQueryRepo {
    async fn get_balance(&self, account_id: &AccountId) -> Result<AccountBalance, AppError> {
      Ok(AccountBalance {
        account_id: *account_id,
        balance: self.balance,
      })
    }
  }

  #[tokio::test]
  async fn get_balance_returns_correct_value() {
    let repo = Arc::new(MockBalanceQueryRepo { balance: 1250 });
    let query = GetBalanceQuery::new(repo);

    let account_id = AccountId(Uuid::now_v7());
    let result = query.execute(account_id).await;

    assert!(result.is_ok());
    let balance = result.unwrap();
    assert_eq!(balance.account_id, account_id);
    assert_eq!(balance.balance, 1250);
  }
}
