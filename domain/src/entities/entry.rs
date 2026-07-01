use crate::value_objects::account::AccountId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum EntryType {
  Debit,
  Credit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryLine {
  pub account_id: AccountId,
  pub amount: u64,
  pub operation: EntryType,
}
