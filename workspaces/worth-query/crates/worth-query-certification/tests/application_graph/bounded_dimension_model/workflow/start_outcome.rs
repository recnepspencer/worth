use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowInstanceStartOutcome,
    WorthQueryWorkflowInstancePreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationUncommitted,
};

type StartResult =
    Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial>;

/// The start named a definition its branch has since superseded; the
/// definition the branch holds current instead is returned.
pub fn expect_superseded_start(result: StartResult) -> PublishedWorkflowDefinitionRef {
    match result.expect("the workflow start must reach commit") {
        WorkflowInstanceStartOutcome::Superseded(start) => start.current().clone(),
        unexpected => panic!("expected a superseded workflow start, got {unexpected:?}"),
    }
}

/// The start named a definition of a retired lineage.
pub fn expect_retired_start(result: StartResult) {
    match result.expect("the workflow start must reach commit") {
        WorkflowInstanceStartOutcome::Retired(_) => {}
        unexpected => panic!("expected a retired workflow start, got {unexpected:?}"),
    }
}

/// The start's lineage already holds its installed number of live instances.
pub fn expect_capacity_refused(result: StartResult) {
    match result.expect("the workflow start must reach commit") {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
                kind: WorthQueryApplicationAttemptDenialKind::WorkflowLineageCapacityUnavailable,
            },
        ),
        unexpected => panic!("a full lineage must refuse a new start: {unexpected:?}"),
    }
}
