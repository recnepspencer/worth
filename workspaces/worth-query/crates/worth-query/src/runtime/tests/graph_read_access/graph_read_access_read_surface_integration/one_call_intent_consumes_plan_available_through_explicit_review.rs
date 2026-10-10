//! one call intent consumes plan available through explicit review.
use super::*;

#[test]
fn one_call_intent_consumes_plan_available_through_explicit_review() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.read-surface.helper-honesty",
    );
    seed_hostile_frontier_graph(
        request_execution,
        &mut workspace,
        "read-surface-helper-honesty",
    );
    let family = graph_access_family(&mut workspace, "read-surface-helper-honesty");
    let (reviewed_plan_digest, reviewed_admission_digest) = {
        let review = workspace
            .read_family_intent(&family)
            .review()
            .expect("read review should be admitted");
        let planned = review
            .graph_read_access_plan()
            .expect("review should expose admitted graph access plan");
        (
            planned.digest().to_string(),
            planned.admission().digest().to_string(),
        )
    };
    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("one-call intent should consume admitted access plan");
    let summary = result
        .receipt()
        .graph_read_access_summary()
        .expect("receipt should expose access summary");
    let counters = result
        .receipt()
        .graph_read_access_complexity_counters()
        .expect("receipt should expose access complexity counters");

    assert_eq!(summary.plan_digest(), reviewed_plan_digest);
    assert_eq!(summary.admission_digest(), reviewed_admission_digest);
    assert_success_counters_are_executor_observed(counters);
    assert_eq!(
        counters.planned_access_step_count(),
        counters.consumed_access_step_count()
    );
    assert!(
        result
            .receipt()
            .graph_read_access_plan_consumption()
            .expect("receipt should expose plan consumption")
            .execution_counters()
            .materialized_row_count()
            > 0
    );
}
