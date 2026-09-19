use std::time::Duration;

use axum::extract::{rejection::JsonRejection, State};
use axum::http::StatusCode;
use axum::Json;
use bank_domain::estate::{DeathNoticeId, EstateAction, EstateCaseId};
use bank_domain::model::BankPrincipalId;
use bank_domain::proposals::BankIdempotencyKey;

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind, BankHttpEstateNotificationOutcome,
    BankHttpEstateNotificationRequest, BankHttpNextAction, BankHttpRecoveryInspectionOutcome,
    BankHttpRecoveryRequest, BankHttpRecoverySafeRetryOutcome,
};
use super::recovery_executor::{
    AdmittedBankHttpNotificationRequest, AdmittedBankHttpRecoveryRequest,
};
use super::recovery_registry::BankHttpRecoveryRegistry;
use super::request_admission::UnadmittedBankHttpControls;
use super::routes::{response_status, BankHttpRouteState};

pub(super) async fn notify_death(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpEstateNotificationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpEstateNotificationOutcome>) {
    let request = match request {
        Ok(Json(request)) => request,
        Err(_) => return notification_response(notification_denied(None, malformed())),
    };
    let admitted = match admit_notification(request, state.maximum_deadline) {
        Ok(admitted) => admitted,
        Err(outcome) => return notification_response(outcome),
    };
    notification_response(state.recovery.notify(admitted).await)
}

pub(super) async fn inspect(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpRecoveryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpRecoveryInspectionOutcome>) {
    let request = match request {
        Ok(Json(request)) => request,
        Err(_) => return inspection_response(inspection_denied(None, malformed())),
    };
    let admitted = match admit_recovery(request, state.maximum_deadline) {
        Ok(admitted) => admitted,
        Err((request_id, denial)) => {
            return inspection_response(inspection_denied(request_id, denial));
        }
    };
    inspection_response(state.recovery.inspect(admitted).await)
}

pub(super) async fn safe_retry(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpRecoveryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpRecoverySafeRetryOutcome>) {
    let request = match request {
        Ok(Json(request)) => request,
        Err(_) => return safe_retry_response(safe_retry_denied(None, malformed())),
    };
    let admitted = match admit_recovery(request, state.maximum_deadline) {
        Ok(admitted) => admitted,
        Err((request_id, denial)) => {
            return safe_retry_response(safe_retry_denied(request_id, denial));
        }
    };
    safe_retry_response(state.recovery.safe_retry(admitted).await)
}

fn admit_notification(
    request: BankHttpEstateNotificationRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedBankHttpNotificationRequest, BankHttpEstateNotificationOutcome> {
    let controls = UnadmittedBankHttpControls {
        protocol: request.protocol,
        request_id: request.request_id,
        deadline_milliseconds: request.controls.deadline_milliseconds,
    }
    .admit(maximum_deadline)
    .map_err(|rejected| notification_denied(rejected.request_id, rejected.denial))?;
    let action = parse_action(&request.estate, &request.notice, &request.subject)
        .ok_or_else(|| notification_denied(Some(controls.request_id.clone()), malformed()))?;
    let idempotency_key = BankIdempotencyKey::new(request.idempotency_key)
        .map_err(|_| notification_denied(Some(controls.request_id.clone()), malformed()))?;
    Ok(AdmittedBankHttpNotificationRequest {
        request_id: controls.request_id,
        credential: request.credential,
        idempotency_key,
        action,
        deadline: controls.deadline,
    })
}

fn admit_recovery(
    request: BankHttpRecoveryRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedBankHttpRecoveryRequest, (Option<String>, BankHttpDenial)> {
    let controls = UnadmittedBankHttpControls {
        protocol: request.protocol,
        request_id: request.request_id,
        deadline_milliseconds: request.controls.deadline_milliseconds,
    }
    .admit(maximum_deadline)
    .map_err(|rejected| (rejected.request_id, rejected.denial))?;
    if !valid_recovery_token(&request.recovery) {
        return Err((Some(controls.request_id), malformed()));
    }
    Ok(AdmittedBankHttpRecoveryRequest {
        request_id: controls.request_id,
        credential: request.credential,
        token: request.recovery,
        deadline: controls.deadline,
    })
}

fn parse_action(estate: &str, notice: &str, subject: &str) -> Option<EstateAction> {
    Some(EstateAction::NotifyDeath {
        estate: EstateCaseId::parse_canonical_text(estate)?,
        notice: DeathNoticeId::parse_canonical_text(notice)?,
        subject: BankPrincipalId::parse_canonical_text(subject)?,
    })
}

pub(super) fn valid_recovery_token(token: &str) -> bool {
    BankHttpRecoveryRegistry::recognizes_token(token)
}

fn notification_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpEstateNotificationOutcome {
    BankHttpEstateNotificationOutcome::Denied { request_id, denial }
}

fn inspection_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpRecoveryInspectionOutcome {
    BankHttpRecoveryInspectionOutcome::Denied { request_id, denial }
}

fn safe_retry_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpRecoverySafeRetryOutcome {
    BankHttpRecoverySafeRetryOutcome::Denied { request_id, denial }
}

fn notification_response(
    outcome: BankHttpEstateNotificationOutcome,
) -> (StatusCode, Json<BankHttpEstateNotificationOutcome>) {
    let status = match &outcome {
        BankHttpEstateNotificationOutcome::Applied { .. } => StatusCode::OK,
        BankHttpEstateNotificationOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

fn inspection_response(
    outcome: BankHttpRecoveryInspectionOutcome,
) -> (StatusCode, Json<BankHttpRecoveryInspectionOutcome>) {
    let status = match &outcome {
        BankHttpRecoveryInspectionOutcome::Inspected { .. } => StatusCode::OK,
        BankHttpRecoveryInspectionOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

fn safe_retry_response(
    outcome: BankHttpRecoverySafeRetryOutcome,
) -> (StatusCode, Json<BankHttpRecoverySafeRetryOutcome>) {
    let status = match &outcome {
        BankHttpRecoverySafeRetryOutcome::Applied { .. } => StatusCode::OK,
        BankHttpRecoverySafeRetryOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

pub(super) const fn malformed() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::MalformedRequest,
        BankHttpNextAction::CorrectRequest,
    )
}
