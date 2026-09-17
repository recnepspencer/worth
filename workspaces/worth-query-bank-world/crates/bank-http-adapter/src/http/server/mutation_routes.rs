use std::time::Duration;

use axum::extract::{rejection::JsonRejection, State};
use axum::http::StatusCode;
use axum::Json;
use bank_domain::proposals::BankIdempotencyKey;

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind, BankHttpMutationFailureKind, BankHttpMutationOperation,
    BankHttpMutationOutcome, BankHttpMutationRequest, BankHttpNextAction,
};
use super::mutation_application::{
    parse_deposit, parse_send_money, parse_withdraw, AdmittedBankHttpMutation,
    AdmittedBankHttpMutationRequest,
};
use super::request_admission::UnadmittedBankHttpControls;
use super::routes::{response_status, BankHttpRouteState};

pub(super) async fn mutate(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpMutationRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpMutationOutcome>) {
    let request = match request {
        Ok(Json(request)) => request,
        Err(_) => return response(malformed(None)),
    };
    let admitted = match admit_mutation(request, state.maximum_deadline) {
        Ok(admitted) => admitted,
        Err(outcome) => return response(outcome),
    };
    response(state.queue.execute_mutation(admitted).await)
}

fn admit_mutation(
    request: BankHttpMutationRequest,
    maximum_deadline: Duration,
) -> Result<AdmittedBankHttpMutationRequest, BankHttpMutationOutcome> {
    let controls = UnadmittedBankHttpControls {
        protocol: request.protocol,
        request_id: request.request_id,
        deadline_milliseconds: request.controls.deadline_milliseconds,
    }
    .admit(maximum_deadline)
    .map_err(|rejected| denied(rejected.request_id, rejected.denial))?;
    let operation = parse_operation(request.operation)
        .ok_or_else(|| malformed(Some(controls.request_id.clone())))?;
    let idempotency_key = BankIdempotencyKey::new(request.idempotency_key)
        .map_err(|_| malformed(Some(controls.request_id.clone())))?;
    Ok(AdmittedBankHttpMutationRequest {
        request_id: controls.request_id,
        credential: request.credential,
        idempotency_key,
        operation,
        deadline: controls.deadline,
    })
}

fn parse_operation(operation: BankHttpMutationOperation) -> Option<AdmittedBankHttpMutation> {
    match operation {
        BankHttpMutationOperation::Deposit {
            institution,
            account,
            amount_minor_units,
        } => parse_deposit(&institution, &account, amount_minor_units),
        BankHttpMutationOperation::Withdraw {
            institution,
            account,
            amount_minor_units,
        } => parse_withdraw(&institution, &account, amount_minor_units),
        BankHttpMutationOperation::SendMoney {
            from,
            recipient,
            amount_minor_units,
        } => parse_send_money(&from, &recipient, amount_minor_units),
    }
}

fn malformed(request_id: Option<String>) -> BankHttpMutationOutcome {
    denied(
        request_id,
        BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
    )
}

fn denied(request_id: Option<String>, denial: BankHttpDenial) -> BankHttpMutationOutcome {
    BankHttpMutationOutcome::NotApplied {
        request_id,
        failure: match denial.kind {
            BankHttpDenialKind::Cancelled => BankHttpMutationFailureKind::Cancelled,
            BankHttpDenialKind::DeadlineExceeded => BankHttpMutationFailureKind::DeadlineExceeded,
            BankHttpDenialKind::Stale => BankHttpMutationFailureKind::Stale,
            _ => BankHttpMutationFailureKind::Aborted,
        },
        stale_fact_count: None,
        provider_recovery: None,
        denial,
    }
}

fn response(outcome: BankHttpMutationOutcome) -> (StatusCode, Json<BankHttpMutationOutcome>) {
    let status = match &outcome {
        BankHttpMutationOutcome::Applied { .. } => StatusCode::OK,
        BankHttpMutationOutcome::NotApplied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}
