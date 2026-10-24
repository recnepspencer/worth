use super::*;
use crate::protocol::{
    BankUserNodeCorrectionHistoryOutcome, BankUserNodeCorrectionHistoryRequest,
    BankUserNodeCorrectionHistoryResumeRequest,
};
pub(super) fn router() -> Router<UserNodeState> {
    Router::new()
        .route("/v1/corrections/history", post(page))
        .route("/v1/corrections/history/resume", post(resume))
}
async fn page(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeCorrectionHistoryRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeCorrectionHistoryOutcome>) {
    let Ok(Json(request)) = request else {
        return response(BankUserNodeCorrectionHistoryOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return response(BankUserNodeCorrectionHistoryOutcome::Denied {
            denial: saturated(),
        });
    };
    response(state.session.correction_history(request).await)
}
async fn resume(
    State(state): State<UserNodeState>,
    request: Result<Json<BankUserNodeCorrectionHistoryResumeRequest>, JsonRejection>,
) -> (StatusCode, Json<BankUserNodeCorrectionHistoryOutcome>) {
    let Ok(Json(request)) = request else {
        return response(BankUserNodeCorrectionHistoryOutcome::Denied {
            denial: malformed(),
        });
    };
    let Ok(_permit) = Arc::clone(&state.requests).try_acquire_owned() else {
        return response(BankUserNodeCorrectionHistoryOutcome::Denied {
            denial: saturated(),
        });
    };
    response(state.session.resume_correction_history(request).await)
}
fn response(
    outcome: BankUserNodeCorrectionHistoryOutcome,
) -> (StatusCode, Json<BankUserNodeCorrectionHistoryOutcome>) {
    let status = match &outcome {
        BankUserNodeCorrectionHistoryOutcome::Forwarded { .. } => StatusCode::OK,
        BankUserNodeCorrectionHistoryOutcome::Denied { denial } => node_denial_status(*denial),
    };
    (status, Json(outcome))
}
