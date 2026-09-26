use worth_query_host::facade::application_entry::{
    WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome,
};

type StartResult =
    Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>;

/// The start reached commit comparison and found a fact it read had moved.
pub fn expect_stale_start(result: StartResult) {
    match result.expect("the workflow start must reach commit comparison") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(
            stale,
        )) => assert!(stale.stale_fact_count() > 0),
        unexpected => panic!("expected stale workflow instance start, got {unexpected:?}"),
    }
}

/// The start's lineage already holds its installed number of live instances.
pub fn expect_capacity_refused(result: StartResult) {
    match result.expect("the workflow start must reach commit") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationCommitOutcome::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
                kind: WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCapacityUnavailable,
            },
        ),
        unexpected => panic!("a full lineage must refuse a new start: {unexpected:?}"),
    }
}
