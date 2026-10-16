//! Native publication and independent grant-revocation stimuli for the public read court.
use super::*;
pub(super) fn publish(application: &GovernedRuntime) -> PublishedWorkflowDefinitionRef {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
        "governed-original-intent",
        super::super::super::definition_limits(),
    )
    .unwrap();
    let terminal = builder.terminal("done").unwrap();
    builder.start(&terminal);
    let definition = builder.finish().unwrap().validate().unwrap();
    publish_definition(
        application,
        definition,
        WorkflowDefinitionExpectedPredecessor::Absent,
        91_500,
    )
}
pub(super) fn publish_definition(
    application: &GovernedRuntime,
    definition: worth_query_host::facade::declaration::application_program::ValidatedWorkflowDefinition<GovernedDocumentWorkflow>,
    predecessor: WorkflowDefinitionExpectedPredecessor,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    let contract = application
        .workflow_spec()
        .bind_definition(definition)
        .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    match application
        .runtime()
        .request(&principal, &scope)
        .mutate(super::super::super::authoring_intent())
        .without_source()
        .idempotency(&key)
        .prepare_workflow_publication(contract, predecessor)
        .unwrap()
        .execute()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("genuine governed definition must publish: {other:?}"),
    }
}
pub(super) fn prepare(
    application: &GovernedRuntime,
    definition: PublishedWorkflowDefinitionRef,
    key: u64,
) -> WorthQueryWorkflowInstanceStartRequest<
    '_,
    DocumentRetentionSchema,
    WorkflowInstanceStartOperation,
    WorkflowInstanceStartInput,
    Document,
> {
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    application
        .runtime()
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_instance_start(application.vocabulary(), definition)
        .unwrap()
}
pub(super) fn revoke(application: &GovernedRuntime, grant: &str, key: u64) {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    // Independent native grant publication is the stimulus, not an alternative workflow read or command adapter.
    let outcome = runtime
        .with_application_advancement(&scope, |phase| {
            runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(WorkflowGrantStatusIntent {
                    input: WorkflowGrantStatusInput {
                        grant_identity: grant.to_owned(),
                        status: "revoked".to_owned(),
                    },
                })
                .without_source()
                .idempotency(&key)
                .execute_in_program_in_advancement(&phase, application.program_runtime())
        })
        .unwrap()
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "real grant revocation must commit: {outcome:?}"
    );
}
