use super::*;

pub(super) fn published_proposal(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> PublishedWorkflowProposalRef {
    match propose_instance(application, instance, key).expect("proposal must prepare") {
        WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
        other => panic!("expected a published proposal, got {other:?}"),
    }
}
