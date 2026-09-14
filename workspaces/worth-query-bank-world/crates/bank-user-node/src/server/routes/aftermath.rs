use std::sync::Arc;

use axum::extract::{rejection::JsonRejection, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

use crate::protocol::{
    BankUserNodeDenial, BankUserNodeDenialKind, BankUserNodeEstateDisbursementOutcome,
    BankUserNodeEstateDisbursementRequest,
};

use super::super::UserNodeState;
use super::node_denial_status;

pub(super) fn router() -> Router<UserNodeState> {
    Router::new().route("/v1/estate/disburse", post(disburse_estate))
}

async fn disburse_estate(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeEstateDisbursementRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeEstateDisbursementOutcome>) {
    let Ok(Json(request)) = request else {
        return response(BankUserNodeEstateDisbursementOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return response(BankUserNodeEstateDisbursementOutcome::Denied {
            denial: saturated(),
        });
    };
    response(state.session.disburse_estate(request).await)
}

fn response(
    outcome: BankUserNodeEstateDisbursementOutcome,
) -> (StatusCode, Json<BankUserNodeEstateDisbursementOutcome>) {
    let status = match &outcome {
        BankUserNodeEstateDisbursementOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeEstateDisbursementOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}

fn malformed() -> BankUserNodeDenial {
    BankUserNodeDenial::new(BankUserNodeDenialKind::MalformedRequest)
}

fn saturated() -> BankUserNodeDenial {
    BankUserNodeDenial::new(BankUserNodeDenialKind::RequestSaturated)
}
