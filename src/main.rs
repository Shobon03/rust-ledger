use application::use_cases::transfer::TransferUseCase;
use infrastructure::database::json_repo::JsonLedgerRepo;
use presentation::run_server;
use std::sync::Arc;

#[tokio::main]
async fn main() {
  let repo = Arc::new(JsonLedgerRepo::new("db/ledger.custom"));
  let use_case = Arc::new(TransferUseCase::new(repo));

  println!("Server running on port 3000");
  run_server(use_case).await;
}
