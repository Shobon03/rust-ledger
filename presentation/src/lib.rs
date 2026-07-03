pub mod dtos;

use crate::dtos::{BalanceResponseDTO, StatementResponseDTO, TransferRequestDTO};
use application::{
  error::AppError,
  queries::{get_balance::GetBalanceQuery, get_statement::GetStatementQuery},
  use_cases::transfer::{TransferRequest, TransferUseCase},
};
use axum::{
  Json, Router,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
  routing::{get, post},
};
use domain::value_objects::{account::AccountId, idempotency::IdempotencyKeyId};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

struct AppState {
  use_case: Arc<TransferUseCase>,
  balance_query: Arc<GetBalanceQuery>,
  statement_query: Arc<GetStatementQuery>,
}

pub async fn run_server(
  port: i32,
  use_case: Arc<TransferUseCase>,
  balance_query: Arc<GetBalanceQuery>,
  statement_query: Arc<GetStatementQuery>,
) {
  let shared_state = Arc::new(AppState {
    use_case,
    balance_query,
    statement_query,
  });

  let app = Router::new()
    .route("/transfers", post(handle_transfer))
    .route("/accounts/{account_id}/balance", get(get_balance))
    .route("/accounts/{account_id}/transactions", get(get_statement))
    .layer(TraceLayer::new_for_http())
    .with_state(shared_state);

  let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
    .await
    .unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn handle_transfer(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<TransferRequestDTO>,
) -> Result<impl IntoResponse, AppError> {
  let req = TransferRequest {
    idempotency_key: IdempotencyKeyId(payload.idempotency_key),
    from_account: AccountId(payload.from_account),
    to_account: AccountId(payload.to_account),
    amount: payload.amount,
  };

  let transaction_id = state.use_case.execute(req).await?;
  Ok((StatusCode::CREATED, transaction_id.to_string()))
}

async fn get_balance(
  State(state): State<Arc<AppState>>,
  Path(account_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
  let account_balance = state.balance_query.execute(AccountId(account_id)).await?;
  let response_dto = BalanceResponseDTO {
    account_id: account_balance.account_id.0,
    balance: account_balance.balance,
  };

  Ok((StatusCode::OK, Json(response_dto)))
}

async fn get_statement(
  State(state): State<Arc<AppState>>,
  Path(account_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
  let account_statement = state.statement_query.execute(AccountId(account_id)).await?;
  let response_dto = StatementResponseDTO {
    account_id: account_statement.account_id.0,
    transactions: account_statement.transactions,
  };

  Ok((StatusCode::OK, Json(response_dto)))
}
