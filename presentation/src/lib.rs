pub mod dtos;

use crate::dtos::{BalanceResponseDTO, StatementResponseDTO, TransferRequestDTO};
use application::{
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
use uuid::Uuid;

struct AppState {
  use_case: Arc<TransferUseCase>,
  balance_query: Arc<GetBalanceQuery>,
  statement_query: Arc<GetStatementQuery>,
}

pub async fn run_server(
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
    .with_state(shared_state);

  let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn handle_transfer(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<TransferRequestDTO>,
) -> (StatusCode, String) {
  let req = TransferRequest {
    idempotency_key: IdempotencyKeyId(payload.idempotency_key),
    from_account: AccountId(payload.from_account),
    to_account: AccountId(payload.to_account),
    amount: payload.amount,
  };

  match state.use_case.execute(req).await {
    Ok(transaction_id) => (StatusCode::CREATED, transaction_id.to_string()),
    Err(_) => (
      StatusCode::UNPROCESSABLE_ENTITY,
      "Failed to process transfer".to_string(),
    ),
  }
}

async fn get_balance(
  State(state): State<Arc<AppState>>,
  Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
  match state.balance_query.execute(AccountId(account_id)).await {
    Ok(account_balance) => {
      let response_dto = BalanceResponseDTO {
        account_id: account_balance.account_id.0,
        balance: account_balance.balance,
      };

      (StatusCode::OK, Json(response_dto)).into_response()
    }
    Err(_) => (
      StatusCode::UNPROCESSABLE_ENTITY,
      "Failed to process request".to_string(),
    )
      .into_response(),
  }
}

async fn get_statement(
  State(state): State<Arc<AppState>>,
  Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
  match state.statement_query.execute(AccountId(account_id)).await {
    Ok(account_statement) => {
      let response_dto = StatementResponseDTO {
        account_id: account_statement.account_id.0,
        transactions: account_statement.transactions,
      };

      (StatusCode::OK, Json(response_dto)).into_response()
    }
    Err(_) => (
      StatusCode::UNPROCESSABLE_ENTITY,
      "Failed to process request".to_string(),
    )
      .into_response(),
  }
}
