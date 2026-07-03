use application::{
  ports::repository::LedgerRepository,
  queries::get_balance::{BalanceQueryRepository, GetBalanceQuery},
  use_cases::transfer::TransferUseCase,
};
use dotenvy::dotenv;
use infrastructure::database::{json_repo::JsonLedgerRepo, rocksdb_repo::RocksDbLedgerRepo};
use presentation::run_server;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() {
  dotenv().ok();

  let use_json = env::var("SAVE_MODE").unwrap_or_else(|_| "json".to_string()) == "json";

  let (repo_write, repo_read): (
    Arc<dyn LedgerRepository + Send + Sync>,
    Arc<dyn BalanceQueryRepository + Send + Sync>,
  ) = if use_json {
    let json_repo = Arc::new(JsonLedgerRepo::new("db/ledger.custom"));
    (json_repo.clone(), json_repo)
  } else {
    let rocks_repo = Arc::new(RocksDbLedgerRepo::new("db/rocks_ledger"));
    (rocks_repo.clone(), rocks_repo)
  };

  let use_case = Arc::new(TransferUseCase::new(repo_write));
  let balance_query = Arc::new(GetBalanceQuery::new(repo_read));

  println!("Server running on port 3000");
  run_server(use_case, balance_query).await;
}
