use application::{error::AppError, ports::repository::LedgerRepository};
use async_trait::async_trait;
use domain::entities::transaction::Transaction;
use std::{
  fs::{File, OpenOptions},
  io::{BufRead, BufReader, Write},
  sync::Mutex,
};

pub struct JsonLedgerRepo {
  file_path: String,
  lock: Mutex<()>,
}

impl JsonLedgerRepo {
  pub fn new(file_path: &str) -> Self {
    Self {
      file_path: file_path.to_string(),
      lock: Mutex::new(()),
    }
  }
}

#[async_trait]
impl LedgerRepository for JsonLedgerRepo {
  async fn save_transaction(&self, transaction: &Transaction) -> Result<(), AppError> {
    let _guard = self.lock.lock().unwrap();

    if let Ok(file) = File::open(&self.file_path) {
      let reader = BufReader::new(file);

      for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
          if let Ok(saved_tx) = serde_json::from_str::<Transaction>(&line_str) {
            if saved_tx.idempotency_key == transaction.idempotency_key {
              return Err(AppError::IdempotencyConflict(saved_tx.id));
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

    Ok(())
  }
}
