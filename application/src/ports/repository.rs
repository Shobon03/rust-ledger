use crate::error::AppError;
use async_trait::async_trait;
use domain::entities::transaction::Transaction;

#[async_trait]
pub trait LedgerRepository: Send + Sync {
  async fn save_transaction(&self, transaction: &Transaction) -> Result<(), AppError>;
}
