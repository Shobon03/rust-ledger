use crate::database::utils::check_and_create_dir;
use application::{
  error::AppError,
  ports::repository::LedgerRepository,
  queries::{
    get_balance::{AccountBalance, BalanceQueryRepository},
    get_statement::{AccountStatement, StatementQueryRepository},
  },
};
use async_trait::async_trait;
use domain::{
  entities::{entry::EntryType, transaction::Transaction},
  value_objects::{account::AccountId, transaction::TransactionId},
};
use std::{
  collections::HashMap,
  fs::{File, OpenOptions},
  io::{BufRead, BufReader, Write},
  sync::{Mutex, RwLock},
};
use tracing::{debug, info, instrument};

pub struct JsonLedgerRepo {
  file_path: String,
  write_lock: Mutex<()>,
  balances: RwLock<HashMap<AccountId, i64>>,
}

impl JsonLedgerRepo {
  pub fn new(file_path: &str) -> Self {
    check_and_create_dir(&file_path);

    let mut cache_balances = HashMap::new();

    if let Ok(file) = File::open(file_path) {
      let reader = BufReader::new(file);

      for line in reader.lines().map_while(Result::ok) {
        if let Ok(transaction) = serde_json::from_str::<Transaction>(&line) {
          for account in transaction.lines {
            let current_balance = cache_balances.entry(account.account_id).or_insert(0);

            if account.operation == EntryType::Credit {
              *current_balance += account.amount as i64;
            } else {
              *current_balance -= account.amount as i64;
            }
          }
        }
      }
    }

    Self {
      file_path: file_path.to_string(),
      write_lock: Mutex::new(()),
      balances: RwLock::new(cache_balances),
    }
  }
}

#[async_trait]
impl LedgerRepository for JsonLedgerRepo {
  #[instrument(skip(self))]
  async fn save_transaction(&self, transaction: &Transaction) -> Result<TransactionId, AppError> {
    debug!(idempotency_key = %transaction.idempotency_key);

    let _guard = self.write_lock.lock().unwrap();

    if let Ok(file) = File::open(&self.file_path) {
      let reader = BufReader::new(file);

      for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
          if let Ok(saved_transaction) = serde_json::from_str::<Transaction>(&line_str) {
            if saved_transaction.idempotency_key == transaction.idempotency_key {
              return Err(AppError::IdempotencyConflict(saved_transaction.id));
            }
          }
        }
      }
    }

    let serialized = serde_json::to_string(transaction)
      .map_err(|e| AppError::Infrastructure(format!("Failed to serialize: {}", e)))?;

    let mut file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&self.file_path)
      .map_err(|e| AppError::Infrastructure(format!("Failed to open file: {}", e)))?;

    writeln!(file, "{}", serialized)
      .map_err(|e| AppError::Infrastructure(format!("Failed to write file: {}", e)))?;

    let mut cache = self.balances.write().unwrap();
    for account in &transaction.lines {
      let current_balance = cache.entry(account.account_id).or_insert(0);

      if account.operation == EntryType::Credit {
        *current_balance += account.amount as i64;
      } else {
        *current_balance -= account.amount as i64;
      }
    }

    info!(transaction_id = %transaction.id, lines = transaction.lines.len(), "Transaction written successfully on JSON DB");

    Ok(transaction.id)
  }
}

#[async_trait]
impl BalanceQueryRepository for JsonLedgerRepo {
  #[instrument(skip(self))]
  async fn get_balance(&self, account_id: &AccountId) -> Result<AccountBalance, AppError> {
    debug!(account_id = %account_id);

    let _guard = self.balances.read().unwrap();
    let balance = _guard.get(account_id).copied().unwrap_or(0);

    info!(account_id = %account_id, balance = balance, "Balance retrieved from JSON DB");

    Ok(AccountBalance {
      account_id: *account_id,
      balance,
    })
  }
}

#[async_trait]
impl StatementQueryRepository for JsonLedgerRepo {
  #[instrument(skip(self))]
  async fn get_statement(
    &self,
    account_id: &AccountId,
    limit: usize,
    offset: usize,
  ) -> Result<AccountStatement, AppError> {
    debug!(account_id = %account_id, limit = limit, offset = offset);

    let _guard = self.write_lock.lock().unwrap();

    let mut transactions = Vec::new();
    let mut skipped = 0;

    if let Ok(file) = File::open(&self.file_path) {
      let reader = BufReader::new(file);

      for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
          if let Ok(transaction) = serde_json::from_str::<Transaction>(&line_str) {
            if transaction
              .lines
              .iter()
              .any(|line| line.account_id == *account_id)
            {
              if skipped < offset {
                skipped += 1;
                continue;
              }

              if transactions.len() >= limit {
                break;
              }

              transactions.push(transaction);
            }
          }
        }
      }
    }

    info!(account_id = %account_id, transactions_count = transactions.len(), "Statement retrieved from JSON DB");

    Ok(AccountStatement {
      account_id: *account_id,
      transactions,
    })
  }
}
