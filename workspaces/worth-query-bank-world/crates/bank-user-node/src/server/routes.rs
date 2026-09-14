use std::sync::Arc;

use axum::extract::{rejection::JsonRejection, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use super::UserNodeState;
use crate::protocol::{
    BankUserNodeAccountActivityPageOutcome, BankUserNodeAccountActivityPageRequest,
    BankUserNodeAccountActivityResumeRequest, BankUserNodeAccountSummaryOutcome,
    BankUserNodeAccountSummaryRequest, BankUserNodeAuthorizationOutcome, BankUserNodeDenial,
    BankUserNodeDenialKind, BankUserNodeEstateNotificationOutcome,
    BankUserNodeEstateNotificationRequest, BankUserNodeMutationOutcome,
    BankUserNodeMutationRequest, BankUserNodeRecoveryInspectionOutcome,
    BankUserNodeRecoveryRequest,
};

mod aftermath;
mod elevation;
mod live;

#[derive(Deserialize)]
struct AuthorizationCallbackQuery {
    code: String,
    state: String,
}

pub(super) fn router() -> Router<UserNodeState> {
    Router::new()
        .route("/health/ready", get(readiness))
        .route("/session/authorize", post(begin_authorization))
        .route("/session/revoke", post(revoke_authorization))
        .route("/oidc/callback", get(finish_authorization))
        .route("/v1/queries/account-summary", post(account_summary))
        .route(
            "/v1/queries/account-activity/page",
            post(account_activity_page),
        )
        .route(
            "/v1/queries/account-activity/resume",
            post(account_activity_resume),
        )
        .route("/v1/mutations", post(mutate))
        .route("/v1/estate/notify-death", post(notify_death))
        .route("/v1/recovery/inspect", post(inspect_recovery))
        .merge(aftermath::router())
        .merge(elevation::router())
        .merge(live::router())
}

async fn readiness() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ready", "process_id": std::process::id() }))
}

async fn begin_authorization(
    State(state): State<UserNodeState>,
) -> (StatusCode, Json<BankUserNodeAuthorizationOutcome>) {
    let outcome = state.session.begin_authorization().await;
    (authorization_status(&outcome), Json(outcome))
}

async fn finish_authorization(
    State(state): State<UserNodeState>,
    Query(callback): Query<AuthorizationCallbackQuery>,
) -> (StatusCode, Json<BankUserNodeAuthorizationOutcome>) {
    let outcome = state
        .session
        .finish_authorization(callback.code, callback.state)
        .await;
    (authorization_status(&outcome), Json(outcome))
}

async fn revoke_authorization(
    State(state): State<UserNodeState>,
) -> (StatusCode, Json<BankUserNodeAuthorizationOutcome>) {
    let outcome = state.session.revoke_authorization().await;
    (authorization_status(&outcome), Json(outcome))
}

async fn account_summary(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeAccountSummaryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeAccountSummaryOutcome>) {
    let Ok(Json(request)) = request else {
        return summary_response(BankUserNodeAccountSummaryOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return summary_response(BankUserNodeAccountSummaryOutcome::Denied {
            denial: saturated(),
        });
    };
    summary_response(state.session.account_summary(request).await)
}

async fn account_activity_page(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeAccountActivityPageRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeAccountActivityPageOutcome>) {
    let Ok(Json(request)) = request else {
        return activity_page_response(activity_denied(malformed()));
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return activity_page_response(activity_denied(saturated()));
    };
    activity_page_response(state.session.account_activity_page(request).await)
}

async fn account_activity_resume(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeAccountActivityResumeRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeAccountActivityPageOutcome>) {
    let Ok(Json(request)) = request else {
        return activity_page_response(activity_denied(malformed()));
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return activity_page_response(activity_denied(saturated()));
    };
    activity_page_response(state.session.account_activity_resume(request).await)
}

async fn mutate(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeMutationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeMutationOutcome>) {
    let Ok(Json(request)) = request else {
        return mutation_response(BankUserNodeMutationOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return mutation_response(BankUserNodeMutationOutcome::Denied {
            denial: saturated(),
        });
    };
    mutation_response(state.session.mutate(request).await)
}

async fn notify_death(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeEstateNotificationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeEstateNotificationOutcome>) {
    let Ok(Json(request)) = request else {
        return notification_response(BankUserNodeEstateNotificationOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return notification_response(BankUserNodeEstateNotificationOutcome::Denied {
            denial: saturated(),
        });
    };
    notification_response(state.session.notify_estate_death(request).await)
}

async fn inspect_recovery(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeRecoveryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeRecoveryInspectionOutcome>) {
    let Ok(Json(request)) = request else {
        return inspection_response(BankUserNodeRecoveryInspectionOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return inspection_response(BankUserNodeRecoveryInspectionOutcome::Denied {
            denial: saturated(),
        });
    };
    inspection_response(state.session.inspect_recovery(request).await)
}

fn summary_response(
    outcome: BankUserNodeAccountSummaryOutcome,
) -> (StatusCode, Json<BankUserNodeAccountSummaryOutcome>) {
    let status = match &outcome {
        BankUserNodeAccountSummaryOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeAccountSummaryOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}

fn activity_denied(denial: BankUserNodeDenial) -> BankUserNodeAccountActivityPageOutcome {
    BankUserNodeAccountActivityPageOutcome::Denied { denial }
}

fn activity_page_response(
    outcome: BankUserNodeAccountActivityPageOutcome,
) -> (StatusCode, Json<BankUserNodeAccountActivityPageOutcome>) {
    let status = match &outcome {
        BankUserNodeAccountActivityPageOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeAccountActivityPageOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}

fn mutation_response(
    outcome: BankUserNodeMutationOutcome,
) -> (StatusCode, Json<BankUserNodeMutationOutcome>) {
    let status = match &outcome {
        BankUserNodeMutationOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeMutationOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}

fn notification_response(
    outcome: BankUserNodeEstateNotificationOutcome,
) -> (StatusCode, Json<BankUserNodeEstateNotificationOutcome>) {
    let status = match &outcome {
        BankUserNodeEstateNotificationOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeEstateNotificationOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}

fn inspection_response(
    outcome: BankUserNodeRecoveryInspectionOutcome,
) -> (StatusCode, Json<BankUserNodeRecoveryInspectionOutcome>) {
    let status = match &outcome {
        BankUserNodeRecoveryInspectionOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeRecoveryInspectionOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}

fn malformed() -> BankUserNodeDenial {
    BankUserNodeDenial::new(BankUserNodeDenialKind::MalformedRequest)
}

fn saturated() -> BankUserNodeDenial {
    BankUserNodeDenial::new(BankUserNodeDenialKind::RequestSaturated)
}

fn node_denial_response(denial: BankUserNodeDenial) -> Response {
    let status = node_denial_status(denial);
    (status, Json(denial)).into_response()
}

const fn node_denial_status(denial: BankUserNodeDenial) -> StatusCode {
    match denial.kind {
        BankUserNodeDenialKind::MalformedRequest => StatusCode::BAD_REQUEST,
        BankUserNodeDenialKind::NoAuthenticatedSession => StatusCode::UNAUTHORIZED,
        BankUserNodeDenialKind::RequestSaturated => StatusCode::TOO_MANY_REQUESTS,
        BankUserNodeDenialKind::UpstreamDeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        _ => StatusCode::BAD_GATEWAY,
    }
}

const fn authorization_status(outcome: &BankUserNodeAuthorizationOutcome) -> StatusCode {
    match outcome {
        BankUserNodeAuthorizationOutcome::AuthorizationRequired { .. }
        | BankUserNodeAuthorizationOutcome::Authenticated
        | BankUserNodeAuthorizationOutcome::Revoked => StatusCode::OK,
        BankUserNodeAuthorizationOutcome::Denied { denial } => match denial.kind {
            BankUserNodeDenialKind::AuthorizationAlreadyPending => StatusCode::CONFLICT,
            BankUserNodeDenialKind::AuthorizationNotPending
            | BankUserNodeDenialKind::AuthorizationRejected
            | BankUserNodeDenialKind::MalformedRequest => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        },
    }
}
