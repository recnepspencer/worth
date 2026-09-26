//! Definition authoring requests, each issued on the branch that holds its
//! definition.

use super::*;

pub fn publish_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>,
    expected_predecessor: WorkflowDefinitionExpectedPredecessor,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let contract = bind_definition(application, definition);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    // A successor is published where its predecessor is held.
    let branch = match &expected_predecessor {
        WorkflowDefinitionExpectedPredecessor::Published(predecessor) => predecessor.branch(),
        WorkflowDefinitionExpectedPredecessor::Absent => runtime.current_world(),
    };
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_publication(contract, expected_predecessor)
        .map(|request| request.execute())
}

pub fn retire_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionRetirementOutcome,
    WorthQueryWorkflowDefinitionRetirementPreparationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(definition.branch())
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_definition_retirement(application, definition)
        .map(|request| request.execute())
}

/// The authoring request every definition publication and retirement presents.
pub fn authoring_intent() -> WorkflowDefinitionAuthoringIntent {
    WorkflowDefinitionAuthoringIntent {
        input: WorkflowDefinitionAuthoringInput {
            identity: PART_IDENTITY.to_owned(),
            dimension: 8,
        },
    }
}
