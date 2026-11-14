//! Ordinary admitted workflow requests against a Home-owned vocabulary.
use super::*;
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
        .prepare_workflow_advance(application.vocabulary(), instance)
        .unwrap()
        .execute()
}
pub(super) fn propose(
    application: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
    target: &TargetNode,
) -> WorkflowProposalOutcome {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let original = request
        .mutate(document::retention_entry::ReviewedSetRetentionIntent {
            input: document::schema::SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency_owned(1012u64)
        .unwrap_or_else(|_| panic!("original guarded target description"));
    let inputs = WorthQueryWorkflowOperationInputs::new(application.runtime())
        .with_at::<document::programs::ReviewedAction, _>(target, original)
        .unwrap_or_else(|_| panic!("actual installed target action"));
    let outcome = request
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal_with_inputs(application.vocabulary(), instance, &inputs)
        .unwrap()
        .execute();
    assert!(
        matches!(
            &outcome,
            WorkflowProposalOutcome::Published(_) | WorkflowProposalOutcome::RecoveredCommitted(_)
        ),
        "proposal: {outcome:?}"
    );
    outcome
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
            application.vocabulary(),
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
pub(super) fn publish_with(
    application: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    definition: worth_query_decl::facade::application_program::ValidatedWorkflowDefinition<
        GovernedDocumentWorkflow,
    >,
) -> PublishedWorkflowDefinitionRef {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let installed = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .vocabulary()
                .workflow_spec_on(&phase, runtime.current_world())
                .unwrap()
        })
        .unwrap();
    let contract = installed.bind_definition(definition).unwrap();
    match runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&1001u64)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)
        .unwrap()
        .execute()
    {
        WorkflowDefinitionPublicationOutcome::Published(done) => done.definition().clone(),
        other => panic!("definition publication: {other:?}"),
    }
}
pub(super) fn start(
    application: &Home<
        impl worth_query_decl::facade::application_program::ApplicationProgramDefinition<
            document::schema::DocumentRetentionSchema,
        >,
    >,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> (
    PublishedWorkflowInstanceRef,
    WorthQueryWorkflowInstanceDiscoveryIntent<document::schema::DocumentRetentionSchema>,
) {
    let runtime = application.runtime().runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(application.vocabulary(), definition)
        .unwrap();
    let original = prepared.discovery_intent();
    match prepared.execute() {
        WorkflowInstanceStartOutcome::Started(done) => (done.instance().clone(), original),
        other => panic!("instance start: {other:?}"),
    }
}
