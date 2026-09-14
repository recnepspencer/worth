use std::time::Duration;

use axum::extract::{rejection::JsonRejection, DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

use super::super::protocol::{
    BankHttpAccountSummaryOutcome, BankHttpAccountSummaryRequest, BankHttpDenial,
    BankHttpDenialKind, BankHttpNextAction,
};
use super::aftermath_routes::disburse;
use super::application::{denied, AdmittedAccountSummaryRequest};
use super::continuation_executor::BankHttpContinuationExecutor;
use super::continuation_routes::{
    page as account_activity_page, resume as account_activity_resume,
};
use super::elevation_executor::BankHttpElevationExecutor;
use super::elevation_routes::{
    approve_elevation, complete_review, request_elevation, revoke_elevation,
};
use super::live_executor::BankHttpLiveExecutor;
use super::live_routes::account_activity_stream;
use super::mutation_routes::mutate;
use super::queue::BankHttpExecutionQueue;
use super::recovery_executor::BankHttpRecoveryExecutor;
use super::recovery_routes::{inspect as inspect_recovery, notify_death};
use super::request_admission::UnadmittedBankHttpRequestBasis;

#[derive(Clone)]
pub(super) struct BankHttpRouteState {
    pub(super) queue: BankHttpExecutionQueue,
    pub(super) live: BankHttpLiveExecutor,
    pub(super) continuations: BankHttpContinuationExecutor,
    pub(super) recovery: BankHttpRecoveryExecutor,
    pub(super) elevation: BankHttpElevationExecutor,
    pub(super) maximum_deadline: Duration,
}

impl BankHttpRouteState {
    pub(super) const fn new(
        queue: BankHttpExecutionQueue,
        live: BankHttpLiveExecutor,
        continuations: BankHttpContinuationExecutor,
        recovery: BankHttpRecoveryExecutor,
        elevation: BankHttpElevationExecutor,
        maximum_deadline: Duration,
    ) -> Self {
        Self {
            queue,
            live,
            continuations,
            recovery,
            elevation,
            maximum_deadline,
        }
    }
}

pub(super) fn router(state: BankHttpRouteState, maximum_body_bytes: usize) -> Router {
    Router::new()
        .route("/health/ready", get(readiness))
        .route("/v1/queries/account-summary", post(account_summary))
        .route(
            "/v1/queries/account-activity/page",
            post(account_activity_page),
        )
        .route(
            "/v1/queries/account-activity/resume",
            post(account_activity_resume),
        )
        .route("/v1/live/account-activity", post(account_activity_stream))
        .route("/v1/mutations", post(mutate))
        .route("/v1/estate/disburse", post(disburse))
        .route("/v1/estate/elevation/request", post(request_elevation))
        .route("/v1/estate/elevation/approve", post(approve_elevation))
        .route("/v1/estate/elevation/revoke", post(revoke_elevation))
        .route("/v1/estate/elevation/review", post(complete_review))
        .route("/v1/estate/notify-death", post(notify_death))
        .route("/v1/recovery/inspect", post(inspect_recovery))
        .layer(DefaultBodyLimit::max(maximum_body_bytes))
        .with_state(state)
}

async fn readiness() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ready", "protocol": "v1" }))
}

async fn account_summary(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpAccountSummaryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpAccountSummaryOutcome>) {
    let request = match request {
        Ok(Json(request)) => request,
        Err(_) => return response(malformed(None)),
    };
    let admitted = match admit_request(request, state.maximum_deadline) {
        Ok(request) => request,
        Err(rejected) => {
            return response(denied(rejected.request_id, rejected.denial));
        }
    };
    response(state.queue.execute(admitted).await)
}

fn admit_request(
    request: BankHttpAccountSummaryRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedAccountSummaryRequest, super::request_admission::RejectedBankHttpRequest> {
    let basis = UnadmittedBankHttpRequestBasis {
        protocol: request.protocol,
        request_id: request.request_id,
        account: request.account,
        deadline_milliseconds: request.controls.deadline_milliseconds,
    }
    .admit(maximum_deadline)?;
    Ok(AdmittedAccountSummaryRequest {
        request_id: basis.request_id,
        credential: request.credential,
        controls: request.controls,
        account: basis.account,
        deadline: basis.deadline,
    })
}

fn malformed(request_id: Option<String>) -> BankHttpAccountSummaryOutcome {
    denied(
        request_id,
        BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
    )
}

fn response(
    outcome: BankHttpAccountSummaryOutcome,
) -> (StatusCode, Json<BankHttpAccountSummaryOutcome>) {
    let status = match &outcome {
        BankHttpAccountSummaryOutcome::Delivered { .. } => StatusCode::OK,
        BankHttpAccountSummaryOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

pub(super) const fn response_status(kind: BankHttpDenialKind) -> StatusCode {
    match kind {
        BankHttpDenialKind::MalformedRequest | BankHttpDenialKind::UnsupportedProtocol => {
            StatusCode::BAD_REQUEST
        }
        BankHttpDenialKind::Unauthenticated => StatusCode::UNAUTHORIZED,
        BankHttpDenialKind::PermissionDenied => StatusCode::FORBIDDEN,
        BankHttpDenialKind::NotFound => StatusCode::NOT_FOUND,
        BankHttpDenialKind::Cancelled | BankHttpDenialKind::DeadlineExceeded => {
            StatusCode::REQUEST_TIMEOUT
        }
        BankHttpDenialKind::Stale => StatusCode::CONFLICT,
        BankHttpDenialKind::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        BankHttpDenialKind::ResourceExhausted | BankHttpDenialKind::Saturated => {
            StatusCode::TOO_MANY_REQUESTS
        }
        BankHttpDenialKind::InternalDenied => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
