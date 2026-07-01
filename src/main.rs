use application::use_cases::transfer::{TransferRequest, TransferUseCase};
use domain::value_objects::{account::AccountId, idempotency::IdempotencyKeyId};
use infrastructure::database::json_repo::JsonLedgerRepo;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() {
  let workspace_dir = env!("CARGO_MANIFEST_DIR");
  let db_dir = format!("{}/db", workspace_dir);
  let file_path = format!("{}/ledger.json", db_dir);

  let repo = Arc::new(JsonLedgerRepo::new(&file_path));

  let transfer_use_case = TransferUseCase::new(repo);

  let idempotency_key = IdempotencyKeyId(Uuid::now_v7());
  let from_account = AccountId(Uuid::now_v7());
  let to_account = AccountId(Uuid::now_v7());
  let amount = 5000;

  let request = TransferRequest {
    idempotency_key,
    from_account,
    to_account,
    amount,
  };

  println!("Starting processing transaction...");

  match transfer_use_case.execute(request).await {
    Ok(transaction_id) => {
      println!("Transaction processed!");
      println!("Transaction id: {:?}", transaction_id);
      println!("Check file: {}", file_path);
    }
    Err(e) => {
      eprintln!("Failed to process transaction: {:?}", e)
    }
  }
}
