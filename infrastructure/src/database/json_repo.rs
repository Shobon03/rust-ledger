use application::{error::AppError, ports::repository::LedgerRepository};
use async_trait::async_trait;
use domain::entities::transaction::Transaction;
use std::{fs::OpenOptions, io::Write, sync::Mutex};

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
    let serialized = serde_json::to_string(transaction)
      .map_err(|e| AppError::Infrastructure(format!("Failed to serialize: {}", e)))?;

    let _guard = self.lock.lock().unwrap();

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
