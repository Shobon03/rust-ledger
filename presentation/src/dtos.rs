use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TransferRequestDTO {
  pub from_account: Uuid,
  pub to_account: Uuid,
  pub amount: u64,
  pub idempotency_key: Uuid,
}

#[derive(Serialize)]
pub struct BalanceResponseDTO {
  pub account_id: Uuid,
  pub balance: i64,
}
