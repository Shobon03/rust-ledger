use application::{queries::get_balance::GetBalanceQuery, use_cases::transfer::TransferUseCase};
use infrastructure::database::json_repo::JsonLedgerRepo;
use presentation::run_server;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() {
  let repo = if env::var("SAVE_MODE").unwrap_or("json".to_string()) == String::from("json") {
    Arc::new(JsonLedgerRepo::new("db/ledger.custom"))
  } else {
    Arc::new(JsonLedgerRepo::new("db/ledger.custom"))
  };
  let use_case = Arc::new(TransferUseCase::new(repo.clone()));
  let balance_query = Arc::new(GetBalanceQuery::new(repo));

  println!("Server running on port 3000");
  run_server(use_case, balance_query).await;
}
