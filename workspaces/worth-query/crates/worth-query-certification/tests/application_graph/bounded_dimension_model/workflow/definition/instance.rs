use super::*;

pub fn start_instance(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(application, definition)
        .map(|request| request.execute())
}

pub fn advance_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .map(|request| request.execute())
}

pub fn approve_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowApproval,
    proposal: &PublishedWorkflowProposalRef,
    decision: WorkflowApprovalDecision,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let signing = runtime
        .request(&principal, &scope)
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_approval(application, instance, required, proposal, decision)?;
    let Some(authentication_intent) = signing.authentication_intent().cloned() else {
        return signing.execute_replay();
    };
    let event = block_on(application.authentication().authenticate(
        (),
        &principal,
        authentication_intent,
        &scope,
    ))
    .expect("the installed certification factor accepts the exact approval challenge");
    signing.sign(&event).map(|request| request.execute())
}

pub fn propose_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    propose_authoring_instance(application, instance, idempotency)
}

pub fn propose_authoring_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    propose_authoring_instance_with_dimension(application, instance, idempotency, 8)
}

pub fn propose_authoring_instance_with_dimension(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
    dimension: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

pub fn propose_instance_on_branch(
    application: &BoundedDimensionWorkflowRuntime,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

/// Continues `instance` as a successor on the current `target`, at `resume_at`.
pub fn migrate_instance(
    application: &BoundedDimensionWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    target: PublishedWorkflowDefinitionRef,
    resume_at: &str,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_migration(application, instance, target, resume_at)
        .map(|request| request.execute())
}
