use application::{
  ports::repository::LedgerRepository,
  queries::{
    get_balance::{BalanceQueryRepository, GetBalanceQuery},
    get_statement::{GetStatementQuery, StatementQueryRepository},
  },
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

  let (ledger, balance, statement): (
    Arc<dyn LedgerRepository + Send + Sync>,
    Arc<dyn BalanceQueryRepository + Send + Sync>,
    Arc<dyn StatementQueryRepository + Send + Sync>,
  ) = if use_json {
    let json_repo = Arc::new(JsonLedgerRepo::new("db/ledger.custom"));
    (json_repo.clone(), json_repo.clone(), json_repo)
  } else {
    let rocks_repo = Arc::new(RocksDbLedgerRepo::new("db/rocks_ledger"));
    (rocks_repo.clone(), rocks_repo.clone(), rocks_repo)
  };

  let use_case = Arc::new(TransferUseCase::new(ledger));
  let balance_query = Arc::new(GetBalanceQuery::new(balance));
  let statement_query = Arc::new(GetStatementQuery::new(statement));

  println!("Server running on port 3000");
  run_server(use_case, balance_query, statement_query).await;
}
