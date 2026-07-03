use std::sync::Arc;

use crate::database::utils::check_and_create_dir;
use application::{
  error::AppError,
  ports::repository::LedgerRepository,
  queries::get_balance::{AccountBalance, BalanceQueryRepository},
};
use domain::{
  entities::{entry::EntryType, transaction::Transaction},
  value_objects::{account::AccountId, transaction::TransactionId},
};
use rocksdb::{DB, Options, WriteBatch};
use uuid::Uuid;

#[derive(Clone)]
pub struct RocksDbLedgerRepo {
  db: Arc<DB>,
}

impl RocksDbLedgerRepo {
  pub fn new(path: &str) -> Self {
    check_and_create_dir(&path);

    let mut opts = Options::default();
    opts.create_if_missing(true);

    let db = DB::open(&opts, path).expect("Failed to open RocksDB");
    Self { db: Arc::new(db) }
  }
}

#[async_trait::async_trait]
impl LedgerRepository for RocksDbLedgerRepo {
  async fn save_transaction(&self, transaction: &Transaction) -> Result<TransactionId, AppError> {
    let key = format!("id:{}", transaction.idempotency_key);

    if let Ok(Some(existing)) = self.db.get(&key) {
      let transaction_id_str = String::from_utf8(existing).unwrap();
      let transaction_id = TransactionId(Uuid::parse_str(&transaction_id_str).unwrap());
      return Ok(transaction_id);
    }

    let mut batch = WriteBatch::default();

    let transaction_key = format!("tx:{}", transaction.id);
    let transaction_bytes = serde_json::to_vec(transaction).unwrap();
    batch.put(transaction_key.as_bytes(), transaction_bytes);

    batch.put(&key.as_bytes(), transaction.id.to_string().as_bytes());

    for account in &transaction.lines {
      let balance_key = format!("bl:{}", account.account_id.0);

      let current_balance = match self.db.get(balance_key.as_bytes()) {
        Ok(Some(bytes)) => {
          let mut arr = [0u8; 8];
          arr.copy_from_slice(&bytes);
          i64::from_be_bytes(arr)
        }
        _ => 0_i64,
      };

      let new_balance = if account.operation == EntryType::Credit {
        current_balance + account.amount as i64
      } else {
        current_balance - account.amount as i64
      };

      batch.put(balance_key.as_bytes(), new_balance.to_be_bytes());
    }

    self
      .db
      .write(batch)
      .map_err(|e| AppError::Infrastructure(e.to_string()))?;

    Ok(transaction.id)
  }
}

#[async_trait::async_trait]
impl BalanceQueryRepository for RocksDbLedgerRepo {
  async fn get_balance(&self, account_id: &AccountId) -> Result<AccountBalance, AppError> {
    let balance_key = format!("bl:{}", account_id.0);

    let balance = match self.db.get(balance_key.as_bytes()) {
      Ok(Some(bytes)) => {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        i64::from_be_bytes(arr)
      }
      _ => 0_i64,
    };

    Ok(AccountBalance {
      account_id: *account_id,
      balance,
    })
  }
}
