//! An adopted branch runs workflows through the vocabulary installed against
//! the program it now runs, and only that one.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowDefinitionBindingDenial,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPreparationDenial,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationRequestExt, WorthQueryWorkflowDefinitionPublicationPreparationDenial,
};
pub use worth_query_host::facade::application_installation::WorthQueryWorkflowRuntimeBindingDenial;
pub use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome;
pub use worth_query_host::facade::product::WorthQueryProductBranch;
pub use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

pub use crate::facade::certification::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP1,
    schema::DocumentRetentionSchema,
    workflow::{
        advance_instance, authoring_intent, install_workflow_spec, start_instance,
        support_workflow_program, terminal_definition, ReviewedDocumentWorkflow,
    },
};

pub type InstalledSpec =
    WorthQueryInstalledApplicationWorkflowSpec<DocumentRetentionSchema, ReviewedDocumentWorkflow>;

#[path = "workflow_vocabulary/support.rs"]
pub mod support;

pub fn second_revision(
    application: &DocumentWorkflowRuntime,
) -> &worth_query_host::facade::declaration::application_program::ApplicationProgramRevision {
    application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
}

pub fn retire_second_program(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &DocumentWorkflowRuntime,
) {
    let revision = *application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision();
    application
        .retire_program_support(request_execution, &revision)
        .expect("an idle rostered program retires");
}

pub fn fork_sibling(
    application: &DocumentWorkflowRuntime,
    parent: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    application
        .with_host_advancement(|host_phase| {
            application
                .runtime()
                .branches()
                .fork(parent)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("the sibling branch publishes")
        })
        .expect("installed host call opens")
}

pub fn adopt_second_program(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
) {
    let target = *application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("an idle workflow host adopts P1");
    assert!(matches!(
        prepared.publish(runtime, &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

/// Publishes a definition bound to the vocabulary `branch` currently runs.
pub fn publish(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    terminal: &str,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let spec = application
        .with_host_advancement(|host_phase| {
            application
                .vocabulary()
                .workflow_spec_on(&host_phase, branch)
                .expect("workflow branch remains admitted")
        })
        .expect("installed host call opens");
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    publish_bound(application, spec, branch, terminal, idempotency, absent)
}

/// Publishes a definition bound to `spec` on `branch`, whichever program the
/// branch runs.
pub fn publish_bound(
    application: &DocumentWorkflowRuntime,
    spec: &InstalledSpec,
    branch: WorthQueryProductBranch,
    terminal: &str,
    idempotency: u64,
    predecessor: WorkflowDefinitionExpectedPredecessor,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let contract = spec
        .bind_definition(terminal_definition(terminal))
        .expect("the definition binds to the vocabulary");
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_publication(contract, predecessor)
        .map(|request| request.execute())
}

pub fn expect_published(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) -> PublishedWorkflowDefinitionRef {
    match result.expect("the definition prepares") {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the definition did not publish: {other:?}"),
    }
}

pub fn expect_stale_definition_vocabulary(
    result: Result<
        WorkflowDefinitionPublicationOutcome,
        WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    >,
) {
    match result {
        Err(WorthQueryWorkflowDefinitionPublicationPreparationDenial::DefinitionPreparation(
            WorkflowDefinitionPreparationDenial::Binding(
                WorkflowDefinitionBindingDenial::ProgramRevisionChanged,
            ),
        )) => {}
        other => panic!("another program's vocabulary must not publish: {other:?}"),
    }
}
