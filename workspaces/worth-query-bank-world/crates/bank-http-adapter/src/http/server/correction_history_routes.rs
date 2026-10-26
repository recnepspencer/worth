//! Strict scalar history admission delegates to the existing bounded executor.
use super::super::protocol::*;
use super::{
    continuation_executor::correction_history::AdmittedHistoryRequest,
    continuation_registry::BankHttpContinuationRegistry,
    routes::{response_status, BankHttpRouteState},
};
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    Json,
};

pub(super) async fn page(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpCorrectionHistoryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpCorrectionHistoryOutcome>) {
    let Ok(Json(request)) = request else {
        return response(denied(None));
    };
    dispatch(&state, request, None).await
}
pub(super) async fn resume(
    State(state): State<BankHttpRouteState>,
    request: Result<Json<BankHttpCorrectionHistoryResumeRequest>, JsonRejection>,
) -> (StatusCode, Json<BankHttpCorrectionHistoryOutcome>) {
    let Ok(Json(request)) = request else {
        return response(denied(None));
    };
    if !BankHttpContinuationRegistry::recognizes_token(&request.continuation) {
        return response(denied(Some(request.request_id)));
    }
    let token = request.continuation;
    dispatch(
        &state,
        BankHttpCorrectionHistoryRequest {
            protocol: request.protocol,
            request_id: request.request_id,
            credential: request.credential,
            controls: request.controls,
            source: request.source,
        },
        Some(token),
    )
    .await
}
async fn dispatch(
    state: &BankHttpRouteState,
    request: BankHttpCorrectionHistoryRequest,
    continuation: Option<String>,
) -> (StatusCode, Json<BankHttpCorrectionHistoryOutcome>) {
    let controls = request.controls;
    let (Some(page_width), Some(history_work), Some(disclosure)) = (
        std::num::NonZeroUsize::new(controls.maximum_entries),
        std::num::NonZeroUsize::new(controls.maximum_history_work),
        std::num::NonZeroUsize::new(controls.maximum_disclosure_results),
    ) else {
        return response(denied(Some(request.request_id)));
    };
    if page_width.get() == usize::MAX {
        return response(denied(Some(request.request_id)));
    }
    let basis = match (super::request_admission::UnadmittedBankHttpControls {
        protocol: request.protocol,
        request_id: request.request_id,
        deadline_milliseconds: controls.deadline_milliseconds,
    })
    .admit(state.maximum_deadline)
    {
        Ok(basis) => basis,
        Err(error) => {
            return response(BankHttpCorrectionHistoryOutcome::Denied {
                request_id: error.request_id,
                denial: error.denial,
            })
        }
    };
    response(
        state
            .continuations
            .correction_history(AdmittedHistoryRequest {
                request_id: basis.request_id,
                credential: request.credential,
                limits: worth_query_host::facade::application_entry::WorthQueryCorrectionHistoryLimits::new(Some(history_work), disclosure),
                page_width,
                source: request.source,
                deadline: basis.deadline,
                continuation,
            })
            .await,
    )
}
fn denied(request_id: Option<String>) -> BankHttpCorrectionHistoryOutcome {
    BankHttpCorrectionHistoryOutcome::Denied {
        request_id,
        denial: BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
    }
}
fn response(
    outcome: BankHttpCorrectionHistoryOutcome,
) -> (StatusCode, Json<BankHttpCorrectionHistoryOutcome>) {
    let status = match &outcome {
        BankHttpCorrectionHistoryOutcome::Delivered { .. } => StatusCode::OK,
        BankHttpCorrectionHistoryOutcome::Denied { denial, .. } => response_status(denial.kind),
    };
    (status, Json(outcome))
}
