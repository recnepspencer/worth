use crate::runtime::WorthQueryGraphReadAccessAdmissionPosture;

use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    frontier_search_family, projection_only_family, workspace,
};

// Ordinary admission has no inline byte forecast, so a frontier read is not
// pushed into paged streaming; it lowers to an inline plan with no stream.
#[test]
fn frontier_read_admits_inline_plan_without_streaming() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-nine.frontier-inline",
    );
    let family = frontier_search_family(&mut workspace, "phase-nine-frontier-inline");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("frontier read should be reviewable");
    let admission = review
        .graph_read_access_admission()
        .expect("frontier admission evidence should exist");
    let plan = review
        .graph_read_access_plan()
        .expect("frontier read should lower to an admitted plan");

    assert!(admission.is_admitted());
    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
    assert_eq!(plan.execution_strategy(), "inline-indexed-read-execution");
    assert!(plan.streaming_plan().is_none());
}

#[test]
fn ordinary_inline_read_does_not_emit_streaming_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-nine.inline-no-streaming",
    );
    let family = projection_only_family(&mut workspace, "phase-nine-inline-no-streaming");
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("inline read should execute");

    assert!(result.receipt().graph_read_streaming_receipt().is_none());
    assert_eq!(
        result
            .receipt()
            .graph_read_access_plan()
            .expect("inline execution should attach access plan")
            .posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
}
