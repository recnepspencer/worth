//! Exact recovery posture at the receiving typed boundary.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind as Session;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationOneShotDenialKind, WorthQueryApplicationQueryAdmissionDenialKind,
};
#[test]
fn content_inconsistency_is_terminal_in_both_query_receivers() {
    assert_eq!(
        admission_posture(
            WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(
                Session::ContentInconsistency
            )
        ),
        WorthQueryRequiredOutputRecoveryPosture::Terminal
    );
    assert_eq!(
        execution_posture(WorthQueryApplicationOneShotDenialKind::ExecutionRequest(
            Session::ContentInconsistency
        )),
        WorthQueryRequiredOutputRecoveryPosture::Terminal
    );
    for kind in [ExecutionKind::Cancelled, ExecutionKind::DeadlineExceeded] {
        assert_eq!(
            execution_posture(kind),
            WorthQueryRequiredOutputRecoveryPosture::Retryable
        );
    }
}
