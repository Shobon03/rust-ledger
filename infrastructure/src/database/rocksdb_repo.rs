use crate::database::utils::check_and_create_dir;
use application::{
  error::AppError,
  ports::repository::LedgerRepository,
  queries::{
    get_balance::{AccountBalance, BalanceQueryRepository},
    get_statement::{AccountStatement, StatementQueryRepository},
  },
};
use dashmap::DashMap;
use domain::{
  entities::{entry::EntryType, transaction::Transaction},
  error::DomainError,
  value_objects::{
    account::{AccountId, TREASURY_ACCOUNT_ID},
    transaction::TransactionId,
  },
};
use rocksdb::{DB, Direction, IteratorMode, Options, WriteBatch};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument};
use uuid::Uuid;

#[derive(Clone)]
pub struct RocksDbLedgerRepo {
  db: Arc<DB>,
  locks: DashMap<Uuid, Arc<Mutex<()>>>,
}

impl RocksDbLedgerRepo {
  pub fn new(path: &str) -> Self {
    check_and_create_dir(&path);

    let mut opts = Options::default();
    opts.create_if_missing(true);

    let db = DB::open(&opts, path).expect("Failed to open RocksDB");
    Self {
      db: Arc::new(db),
      locks: DashMap::new(),
    }
  }
}

#[async_trait::async_trait]
impl LedgerRepository for RocksDbLedgerRepo {
  #[instrument(skip(self))]
  async fn save_transaction(&self, transaction: &Transaction) -> Result<TransactionId, AppError> {
    debug!(idempotency_key = %transaction.idempotency_key);

    let key = format!("id:{}", transaction.idempotency_key);

    if let Ok(Some(existing)) = self.db.get(&key) {
      let transaction_id_str = String::from_utf8(existing).unwrap();
      let transaction_id = TransactionId(Uuid::parse_str(&transaction_id_str).unwrap());

      info!(transaction_id = %transaction_id, "Idempotency key found, will not save");

      return Ok(transaction_id);
    }

    let mut account_ids: Vec<Uuid> = transaction.lines.iter().map(|l| l.account_id.0).collect();
    account_ids.sort();

    let mut _guards = Vec::new();
    for id in account_ids {
      let lock_arc = self
        .locks
        .entry(id)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();
      _guards.push(lock_arc.lock_owned().await);
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

      if new_balance < 0 && account.account_id.0 != TREASURY_ACCOUNT_ID {
        return Err(AppError::Domain(DomainError::InsufficientFunds(
          account.account_id,
        )));
      }

      batch.put(balance_key.as_bytes(), new_balance.to_be_bytes());

      let idx_key = format!(
        "id_tx:{}:{}:{}",
        account.account_id.0, transaction.timestamp, transaction.id
      );
      batch.put(idx_key.as_bytes(), transaction.id.to_string().as_bytes());
    }

    self
      .db
      .write(batch)
      .map_err(|e| AppError::Infrastructure(e.to_string()))?;

    info!(transaction_id = %transaction.id, lines = transaction.lines.len(), "Transaction written successfully on RocksDB");

    Ok(transaction.id)
  }
}

#[async_trait::async_trait]
impl BalanceQueryRepository for RocksDbLedgerRepo {
  #[instrument(skip(self))]
  async fn get_balance(&self, account_id: &AccountId) -> Result<AccountBalance, AppError> {
    debug!(account_id = %account_id);

    let balance_key = format!("bl:{}", account_id.0);

    let balance = match self.db.get(balance_key.as_bytes()) {
      Ok(Some(bytes)) => {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        i64::from_be_bytes(arr)
      }
      _ => 0_i64,
    };

    info!(account_id = %account_id, balance = balance, "Balance retrieved from RocksDB");

    Ok(AccountBalance {
      account_id: *account_id,
      balance,
    })
  }
}

#[async_trait::async_trait]
impl StatementQueryRepository for RocksDbLedgerRepo {
  #[instrument(skip(self))]
  async fn get_statement(&self, account_id: &AccountId) -> Result<AccountStatement, AppError> {
    debug!(account_id = %account_id);

    let prefix = format!("id_tx:{}:", account_id.0);

    let iter = self
      .db
      .iterator(IteratorMode::From(prefix.as_bytes(), Direction::Forward));

    let mut transactions = Vec::new();
    for item in iter {
      let (key, value) = item.unwrap();
      let key_str = String::from_utf8(key.to_vec()).unwrap();

      if !key_str.starts_with(&prefix) {
        break;
      }

      let tx_id_str = String::from_utf8(value.to_vec()).unwrap();
      let tx_key = format!("tx:{}", tx_id_str);

      if let Ok(Some(tx_bytes)) = self.db.get(tx_key.as_bytes()) {
        let transaction: Transaction = serde_json::from_slice(&tx_bytes).unwrap();
        transactions.push(transaction);
      }
    }

    info!(account_id = %account_id, transactions_count = transactions.len(), "Statement retrieved from RocksDB");

    Ok(AccountStatement {
      account_id: *account_id,
      transactions,
    })
  }
}
