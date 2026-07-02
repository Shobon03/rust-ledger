use application::use_cases::transfer::{TransferRequest, TransferUseCase};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use domain::value_objects::{account::AccountId, idempotency::IdempotencyKeyId};
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Deserialize)]
struct TransferDTO {
  from_account: Uuid,
  to_account: Uuid,
  amount: u64,
  idempotency_key: Uuid,
}

struct AppState {
  use_case: Arc<TransferUseCase>,
}

pub async fn run_server(use_case: Arc<TransferUseCase>) {
  let shared_state = Arc::new(AppState { use_case });

  let app = Router::new()
    .route("/transfers", post(handle_transfer))
    .with_state(shared_state);

  let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn handle_transfer(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<TransferDTO>,
) -> (StatusCode, String) {
  let req = TransferRequest {
    idempotency_key: IdempotencyKeyId(payload.idempotency_key),
    from_account: AccountId(payload.from_account),
    to_account: AccountId(payload.to_account),
    amount: payload.amount,
  };

  match state.use_case.execute(req).await {
    Ok(tx_id) => (StatusCode::CREATED, tx_id.to_string()),
    Err(_) => (
      StatusCode::UNPROCESSABLE_ENTITY,
      "Failed to process transfer".to_string(),
    ),
  }
}
