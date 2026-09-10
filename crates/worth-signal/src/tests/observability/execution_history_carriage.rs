use crate::facade::{
    ChangedRegion, DiagnosticsTier, EvaluationRequestMode, NodeEvaluationResult, OutputChange,
    SignalGraph, SignalRuntimePolicy,
};
use crate::tests::support::version_ab;

const HISTORY_CARRIAGE_CHILD_ENV: &str = "WORTH_SIGNAL_HISTORY_CARRIAGE_CHILD";
const HISTORY_CARRIAGE_TEST_NAME: &str =
    "tests::observability::execution_history_carriage::complete_history_carriage_matches_reconstruction_and_incomplete_coverage_falls_back";

#[test]
fn complete_history_carriage_matches_reconstruction_and_incomplete_coverage_falls_back() {
    if super::isolated_counter_process::run_in_isolated_counter_process(
        HISTORY_CARRIAGE_TEST_NAME,
        HISTORY_CARRIAGE_CHILD_ENV,
    ) {
        return;
    }

    let mut complete = SignalGraph::new();
    complete.set_runtime_policy(SignalRuntimePolicy::development());
    let nodes = (0..70).map(|_| complete.node().build()).collect::<Vec<_>>();
    let plan = complete
        .build_evaluation_plan(&nodes, EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    let before_complete = crate::data::access_counters::snapshot();
    complete
        .execute_prepared_plan(&plan, &(), &|ctx| {
            Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(ctx.node().index() as u64 + 1, 0))
                    .with_output_change(OutputChange::Refreshed)
                    .with_changed_region(ChangedRegion::new(format!(
                        "node-{}",
                        ctx.node().index()
                    ))),
            ))
        })
        .unwrap();
    let complete_access = crate::data::access_counters::snapshot().delta_since(before_complete);
    let carried = complete
        .observe()
        .recent_execution_history_diagnostics()
        .last()
        .cloned()
        .expect("complete execution should retain its carried history");
    let reconstructed = complete
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);

    assert_eq!(carried, reconstructed);
    assert!(carried.nodes.len() < nodes.len());
    assert_eq!(
        complete_access.runtime_artifact_state_reads,
        nodes.len() as u64,
        "complete coverage must consume retained facts without a second graph reconstruction"
    );

    let mut incomplete = SignalGraph::new();
    incomplete.set_runtime_policy(SignalRuntimePolicy::development());
    let executed = incomplete.node().build();
    let _unexecuted = incomplete.node().build();
    let plan = incomplete
        .build_evaluation_plan(&[executed], EvaluationRequestMode::ForceOnDemand)
        .unwrap();
    let before_incomplete = crate::data::access_counters::snapshot();
    incomplete
        .execute_prepared_plan(&plan, &(), &|ctx| {
            Ok(ctx.finish(NodeEvaluationResult::from_version(version_ab(1, 0))))
        })
        .unwrap();
    let incomplete_access = crate::data::access_counters::snapshot().delta_since(before_incomplete);
    let retained_fallback = incomplete
        .observe()
        .recent_execution_history_diagnostics()
        .last()
        .cloned()
        .expect("incomplete execution should retain reconstructed history");
    let reconstructed_fallback = incomplete
        .observe()
        .execution_history_summary(DiagnosticsTier::Development);

    assert_eq!(retained_fallback, reconstructed_fallback);
    assert_eq!(retained_fallback.traced_node_count, 1);
    assert_eq!(
        incomplete_access.runtime_artifact_state_reads, 3,
        "incomplete coverage must return to the two-node reconstruction fallback"
    );
}
