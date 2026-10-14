//! HTTP delivery delegates the same independently reachable owning command entrance.
use super::super::protocol::{
    BankHttpDenialKind, BankHttpMutationOutcome, BankHttpMutationRequest,
};
use super::routes::{response_status, BankHttpRouteState};
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    Json,
};
pub(super) async fn mutate(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpMutationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpMutationOutcome>) {
    let outcome = match request {
        Ok(Json(request)) => state.mutations.execute(request).await,
        Err(_) => super::mutation_command::reject(None, BankHttpDenialKind::MalformedRequest),
    };
    let status = match &outcome {
        BankHttpMutationOutcome::Rejected { denial, .. }
        | BankHttpMutationOutcome::Refused { denial, .. } => response_status(denial.kind),
        BankHttpMutationOutcome::Pending { .. }
        | BankHttpMutationOutcome::OutcomeUnknown { .. } => StatusCode::ACCEPTED,
        _ => StatusCode::OK,
    };
    (status, Json(outcome))
}
