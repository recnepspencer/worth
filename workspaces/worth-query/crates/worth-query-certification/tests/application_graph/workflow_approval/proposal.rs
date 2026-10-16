use super::*;

pub(super) fn published_proposal(
    application: &DocumentWorkflowRuntime<
        crate::document_retention_model::programs::RetentionProgramP0,
        impl worth_query_host::facade::declaration::application_program::ApplicationWorkflowSpec<
            Schema = crate::document_retention_model::schema::DocumentRetentionSchema,
        >,
    >,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> PublishedWorkflowProposalRef {
    match propose_instance(application, instance, key).expect("proposal must prepare") {
        WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
        other => panic!("expected a published proposal, got {other:?}"),
    }
}
