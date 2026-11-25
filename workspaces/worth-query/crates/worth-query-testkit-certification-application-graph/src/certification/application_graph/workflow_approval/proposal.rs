pub use super::*;

pub fn published_proposal<A>(
    application: &A,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> PublishedWorkflowProposalRef
where
    A: worth_query_testkit_document_retention::facade::document_retention_model::host::DocumentWorkflowApplication<
        Program = worth_query_testkit_document_retention::facade::document_retention_model::programs::RetentionProgramP0,
    >,
{
    match propose_instance(application, instance, key).expect("proposal must prepare") {
        WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
        other => panic!("expected a published proposal, got {other:?}"),
    }
}
