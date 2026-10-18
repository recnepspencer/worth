use std::time::Duration;

use axum::extract::{rejection::JsonRejection, State};
use axum::http::StatusCode;
use axum::Json;
use bank_domain::estate::{
    CapabilityGrantId, EmergencyAccessId, EmergencyAccessReason, EstateAction, EstateCaseId,
    MandatoryReviewId, RestrictedBankField,
};
use bank_domain::proposals::BankIdempotencyKey;

use super::super::protocol::{
    BankHttpDenial, BankHttpElevationApprovalOutcome, BankHttpElevationApprovalRequest,
    BankHttpElevationRequest, BankHttpElevationRequestOutcome, BankHttpElevationRevocationOutcome,
    BankHttpElevationRevocationRequest, BankHttpEmergencyAccessReason,
    BankHttpMandatoryReviewOutcome, BankHttpMandatoryReviewRequest, BankHttpRestrictedBankField,
};
use super::elevation_executor::{
    AdmittedBankHttpElevationRequest, AdmittedBankHttpElevationTransition,
};
use super::elevation_registry::BankHttpElevationContext;
use super::request_admission::UnadmittedBankHttpControls;
use super::routes::{response_status, BankHttpRouteState};

pub(super) async fn request_elevation(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpElevationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpElevationRequestOutcome>) {
    let request = match request {
        Ok(Json(request)) => request,
        Err(_) => return request_response(request_denied(None, malformed())),
    };
    let admitted = match admit_request(request, state.maximum_deadline) {
        Ok(admitted) => admitted,
        Err((request_id, denial)) => return request_response(request_denied(request_id, denial)),
    };
    request_response(state.elevation.request(admitted).await)
}

pub(super) async fn approve_elevation(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpElevationApprovalRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpElevationApprovalOutcome>) {
    let admitted = match request {
        Ok(Json(request)) => admit_transition(
            request.protocol,
            request.request_id,
            request.credential,
            request.controls.deadline_milliseconds,
            request.idempotency_key,
            request.elevation,
            state.maximum_deadline,
        ),
        Err(_) => Err((None, malformed())),
    };
    let admitted = match admitted {
        Ok(admitted) => admitted,
        Err((request_id, denial)) => {
            return approval_response(approval_denied(request_id, denial));
        }
    };
    approval_response(state.elevation.approve(admitted).await)
}

pub(super) async fn revoke_elevation(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpElevationRevocationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpElevationRevocationOutcome>) {
    let admitted = match request {
        Ok(Json(request)) => admit_transition(
            request.protocol,
            request.request_id,
            request.credential,
            request.controls.deadline_milliseconds,
            request.idempotency_key,
            request.elevation,
            state.maximum_deadline,
        ),
        Err(_) => Err((None, malformed())),
    };
    let admitted = match admitted {
        Ok(admitted) => admitted,
        Err((request_id, denial)) => {
            return revocation_response(revocation_denied(request_id, denial));
        }
    };
    revocation_response(state.elevation.revoke(admitted).await)
}

pub(super) async fn complete_review(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpMandatoryReviewRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpMandatoryReviewOutcome>) {
    let admitted = match request {
        Ok(Json(request)) => admit_transition(
            request.protocol,
            request.request_id,
            request.credential,
            request.controls.deadline_milliseconds,
            request.idempotency_key,
            request.mandatory_review,
            state.maximum_deadline,
        ),
        Err(_) => Err((None, malformed())),
    };
    let admitted = match admitted {
        Ok(admitted) => admitted,
        Err((request_id, denial)) => return review_response(review_denied(request_id, denial)),
    };
    review_response(state.elevation.review(admitted).await)
}

fn admit_request(
    request: BankHttpElevationRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedBankHttpElevationRequest, (Option<String>, BankHttpDenial)> {
    let controls = UnadmittedBankHttpControls {
        protocol: request.protocol,
        request_id: request.request_id,
        deadline_milliseconds: request.controls.deadline_milliseconds,
    }
    .admit(maximum_deadline)
    .map_err(|rejected| (rejected.request_id, rejected.denial))?;
    let invalid = || (Some(controls.request_id.clone()), malformed());
    let estate = EstateCaseId::parse_canonical_text(&request.estate).ok_or_else(invalid)?;
    let access = EmergencyAccessId::new(request.access).ok_or_else(invalid)?;
    let review = MandatoryReviewId::new(request.mandatory_review).ok_or_else(invalid)?;
    let grant = CapabilityGrantId::new(request.upper_bound_grant).ok_or_else(invalid)?;
    if request.duration_seconds == 0 {
        return Err(invalid());
    }
    let action = EstateAction::RequestEmergencyAccess {
        estate,
        access,
        review,
        grant,
        reason: reason(request.reason),
        field: field(request.field),
        duration: Duration::from_secs(request.duration_seconds),
    };
    let idempotency_key =
        BankIdempotencyKey::new(request.idempotency_key).map_err(|_| invalid())?;
    Ok(AdmittedBankHttpElevationRequest {
        request_id: controls.request_id,
        credential: request.credential,
        idempotency_key,
        action,
        context: BankHttpElevationContext::new(estate, access, review),
        deadline: controls.deadline,
    })
}

fn admit_transition(
    protocol: super::super::protocol::BankHttpProtocolVersion,
    request_id: String,
    credential: super::super::protocol::BankHttpCredential,
    deadline_milliseconds: u64,
    idempotency_key: String,
    token: String,
    maximum_deadline: Duration,
) -> Result<AdmittedBankHttpElevationTransition, (Option<String>, BankHttpDenial)> {
    let controls = UnadmittedBankHttpControls {
        protocol,
        request_id,
        deadline_milliseconds,
    }
    .admit(maximum_deadline)
    .map_err(|rejected| (rejected.request_id, rejected.denial))?;
    if !super::elevation_registry::BankHttpElevationRegistry::recognizes_token(&token) {
        return Err((Some(controls.request_id), malformed()));
    }
    let idempotency_key = BankIdempotencyKey::new(idempotency_key)
        .map_err(|_| (Some(controls.request_id.clone()), malformed()))?;
    Ok(AdmittedBankHttpElevationTransition {
        request_id: controls.request_id,
        credential,
        idempotency_key,
        token,
        deadline: controls.deadline,
    })
}

fn reason(reason: BankHttpEmergencyAccessReason) -> EmergencyAccessReason {
    match reason {
        BankHttpEmergencyAccessReason::PreventImmediateLoss => {
            EmergencyAccessReason::PreventImmediateLoss
        }
        BankHttpEmergencyAccessReason::ProtectVulnerableCustomer => {
            EmergencyAccessReason::ProtectVulnerableCustomer
        }
        BankHttpEmergencyAccessReason::MeetLegalDeadline => {
            EmergencyAccessReason::MeetLegalDeadline
        }
    }
}

fn field(field: BankHttpRestrictedBankField) -> RestrictedBankField {
    match field {
        BankHttpRestrictedBankField::CustomerIdentity => RestrictedBankField::CustomerIdentity,
        BankHttpRestrictedBankField::BeneficiaryIdentity => {
            RestrictedBankField::BeneficiaryIdentity
        }
        BankHttpRestrictedBankField::LegalDocument => RestrictedBankField::LegalDocument,
        BankHttpRestrictedBankField::AccountDetails => RestrictedBankField::AccountDetails,
        BankHttpRestrictedBankField::PostingHistory => RestrictedBankField::PostingHistory,
        BankHttpRestrictedBankField::AuditTrail => RestrictedBankField::AuditTrail,
        BankHttpRestrictedBankField::GovernanceMetadata => RestrictedBankField::GovernanceMetadata,
        BankHttpRestrictedBankField::EmergencyAccessActivity => {
            RestrictedBankField::EmergencyAccessActivity
        }
    }
}

fn request_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpElevationRequestOutcome {
    BankHttpElevationRequestOutcome::Denied { request_id, denial }
}

fn approval_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpElevationApprovalOutcome {
    BankHttpElevationApprovalOutcome::Denied { request_id, denial }
}

fn revocation_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpElevationRevocationOutcome {
    BankHttpElevationRevocationOutcome::Denied { request_id, denial }
}

fn review_denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpMandatoryReviewOutcome {
    BankHttpMandatoryReviewOutcome::Denied { request_id, denial }
}

fn request_response(
    outcome: BankHttpElevationRequestOutcome,
) -> (StatusCode, Json<BankHttpElevationRequestOutcome>) {
    let status = match &outcome {
        BankHttpElevationRequestOutcome::Requested { .. } => StatusCode::OK,
        BankHttpElevationRequestOutcome::CleanupRequired { .. }
        | BankHttpElevationRequestOutcome::OutcomeUnknown { .. } => StatusCode::ACCEPTED,
        BankHttpElevationRequestOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

fn approval_response(
    outcome: BankHttpElevationApprovalOutcome,
) -> (StatusCode, Json<BankHttpElevationApprovalOutcome>) {
    let status = match &outcome {
        BankHttpElevationApprovalOutcome::Approved { .. } => StatusCode::OK,
        BankHttpElevationApprovalOutcome::CleanupRequired { .. }
        | BankHttpElevationApprovalOutcome::OutcomeUnknown { .. } => StatusCode::ACCEPTED,
        BankHttpElevationApprovalOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

fn revocation_response(
    outcome: BankHttpElevationRevocationOutcome,
) -> (StatusCode, Json<BankHttpElevationRevocationOutcome>) {
    let status = match &outcome {
        BankHttpElevationRevocationOutcome::Closed { .. } => StatusCode::OK,
        BankHttpElevationRevocationOutcome::CleanupRequired { .. }
        | BankHttpElevationRevocationOutcome::OutcomeUnknown { .. } => StatusCode::ACCEPTED,
        BankHttpElevationRevocationOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

fn review_response(
    outcome: BankHttpMandatoryReviewOutcome,
) -> (StatusCode, Json<BankHttpMandatoryReviewOutcome>) {
    let status = match &outcome {
        BankHttpMandatoryReviewOutcome::Reviewed { .. } => StatusCode::OK,
        BankHttpMandatoryReviewOutcome::CleanupRequired { .. }
        | BankHttpMandatoryReviewOutcome::OutcomeUnknown { .. } => StatusCode::ACCEPTED,
        BankHttpMandatoryReviewOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}

fn malformed() -> BankHttpDenial {
    BankHttpDenial::new(
        super::super::protocol::BankHttpDenialKind::MalformedRequest,
        super::super::protocol::BankHttpNextAction::CorrectRequest,
    )
}
