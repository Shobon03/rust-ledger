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
use tracing::info;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
  dotenv().ok();

  let file_appender = tracing_appender::rolling::daily("logs", "ledger.log");
  let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

  tracing_subscriber::registry()
    .with(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,rust_ledger=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .with(
      tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking_writer),
    )
    .init();

  info!("Starting ledger server...");

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

  let port = 3000;

  info!("Server running on port {}", port);
  run_server(port, use_case, balance_query, statement_query).await;
}
