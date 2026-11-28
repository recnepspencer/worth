//! Ordinary admitted workflow requests against a Home-owned vocabulary.
use super::*;
pub(super) use document::workflow::home_transition::{propose, publish_with, start};
pub(super) fn advance(
    application: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> WorkflowProgressOutcome {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(
            application
                .workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            instance,
        )
        .unwrap()
        .execute()
}
pub(super) fn approve(
    application: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    authentication: &CertificationAuthenticationOwner,
    instance: PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    idempotency: u64,
) -> WorkflowProgressOutcome {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_reviewer(runtime.installed_schema(), &scope);
    let signing = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_approval(
            application
                .workflow_vocabulary::<GovernedDocumentWorkflow>()
                .unwrap(),
            instance,
            required,
            proposal,
            WorkflowApprovalDecision::Approve,
        )
        .unwrap();
    let Some(authentication_intent) = signing.authentication_intent().cloned() else {
        return signing.execute_replay().unwrap();
    };
    let event =
        block_on(authentication.authenticate((), &principal, authentication_intent, &scope))
            .expect("the installed certification factor accepts the exact approval challenge");
    signing.sign(&event).unwrap().execute()
}
pub(super) fn publish(
    application: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
) -> (PublishedWorkflowDefinitionRef, TargetNode) {
    let (definition, target) = definition::reviewed();
    (publish_with(application, definition), target)
}
