use super::*;

pub fn start_instance(
    application: &DocumentWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(definition.branch())
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(application, definition)
        .map(|request| request.execute())
}

pub fn advance_instance(
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    advance_instance_on(application, instance.branch(), instance, idempotency)
}

/// Advances `instance` through a request selected on `branch`, which may be
/// a branch the instance does not live on.
pub fn advance_instance_on(
    application: &DocumentWorkflowRuntime,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(application, instance)
        .map(|request| request.execute())
}

pub fn approve_instance(
    application: &DocumentWorkflowRuntime,
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
        .on_branch(instance.branch())
        .mutate(WorkflowApprovalIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
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
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    propose_authoring_instance(application, instance, idempotency)
}

pub fn propose_authoring_instance(
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    propose_authoring_instance_with_retention(application, instance, idempotency, 8)
}

pub fn propose_authoring_instance_with_retention(
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
    retention_days: u64,
) -> Result<WorkflowProposalOutcome, WorthQueryWorkflowProposalPreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

pub fn propose_instance_on_branch(
    application: &DocumentWorkflowRuntime,
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
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_proposal(application, instance)
        .map(|request| request.execute())
}

/// Continues `instance` as a successor on the current `target`, at `resume_at`.
pub fn migrate_instance(
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    target: PublishedWorkflowDefinitionRef,
    resume_at: &str,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_migration(application, instance, target, resume_at)
        .map(|request| request.execute())
}

/// Continues the copy `fork` holds of `instance`, started on another branch,
/// as a successor on the fork's current `target`, at `resume_at`.
pub fn continue_on_fork(
    application: &DocumentWorkflowRuntime,
    fork: worth_query_host::facade::product::WorthQueryProductBranch,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    target: PublishedWorkflowDefinitionRef,
    resume_at: &str,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstancePreparationDenial> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(fork)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_fork_continuation(application, instance, target, resume_at)
        .map(|request| request.execute())
}

/// Prepares the cancellation of `instance` on its own branch and returns its
/// execution, so a test can race it against another commit.
pub fn prepare_cancellation(
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<
    impl FnOnce() -> worth_query_host::facade::application_entry::WorkflowInstanceCancellationOutcome
        + '_,
    WorthQueryWorkflowInstancePreparationDenial,
> {
    prepare_cancellation_on(application, instance.branch(), instance, idempotency)
}

/// Prepares the cancellation of `instance` on `branch`, which may be another
/// branch than the instance's own.
pub fn prepare_cancellation_on(
    application: &DocumentWorkflowRuntime,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<
    impl FnOnce() -> worth_query_host::facade::application_entry::WorkflowInstanceCancellationOutcome
        + '_,
    WorthQueryWorkflowInstancePreparationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_cancellation(application, instance)
        .map(|request| move || request.execute())
}

pub fn cancel_instance(
    application: &DocumentWorkflowRuntime,
    instance: worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<
    worth_query_host::facade::application_entry::WorkflowInstanceCancellationOutcome,
    WorthQueryWorkflowInstancePreparationDenial,
> {
    prepare_cancellation(application, instance, idempotency).map(|execute| execute())
}
