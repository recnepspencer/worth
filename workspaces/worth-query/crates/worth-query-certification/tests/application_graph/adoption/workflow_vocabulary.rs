//! An adopted branch runs workflows through the vocabulary installed against
//! the program it now runs, and only that one.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, WorkflowDefinitionBindingDenial,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPreparationDenial,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationRequestExt, WorthQueryWorkflowDefinitionPublicationPreparationDenial,
};
use worth_query_host::facade::application_installation::WorthQueryWorkflowRuntimeBindingDenial;
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use crate::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP1,
    schema::DocumentRetentionSchema,
    workflow::{
        advance_instance, authoring_intent, install_workflow_spec, start_instance,
        support_workflow_program, terminal_definition, ReviewedDocumentWorkflow,
    },
};

type InstalledSpec =
    WorthQueryInstalledApplicationWorkflowSpec<DocumentRetentionSchema, ReviewedDocumentWorkflow>;

#[test]
fn an_adopted_branch_runs_workflows_through_its_new_program_vocabulary() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);
    let main = application.current_world();
    let initial = application.workflow_spec().program_revision();
    assert_eq!(
        application
            .vocabulary()
            .workflow_spec_on(main)
            .program_revision(),
        initial,
        "a supported vocabulary serves no branch until the branch adopts its program",
    );
    adopt_second_program(&application, main);

    let second = application.vocabulary().workflow_spec_on(main);
    assert_eq!(second.program_revision(), second_revision(&application));
    assert_ne!(second.program_revision(), initial);

    let definition = expect_published(publish(&application, main, "completed", 84_000));
    let instance = match start_instance(&application, definition, 84_001)
        .expect("the P1 instance prepares on the adopted branch")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the P1 instance did not start: {other:?}"),
    };
    assert_eq!(instance.program_revision(), second.program_revision());
    match advance_instance(&application, instance, 84_002)
        .expect("the P1 advance prepares on the adopted branch")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
            assert!(transition.terminal());
        }
        other => panic!("the P1 instance did not complete: {other:?}"),
    }

    // A definition bound to the program the branch left cannot publish there.
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    let initial_spec = application.workflow_spec();
    let stale = publish_bound(&application, initial_spec, main, "stale", 84_003, absent);
    expect_stale_definition_vocabulary(stale);
}

#[test]
fn a_sibling_keeps_its_program_vocabulary_after_its_parent_adopts() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);
    let main = application.current_world();
    let sibling = fork_sibling(&application, main);
    adopt_second_program(&application, main);

    let vocabulary = application.vocabulary();
    assert_eq!(
        vocabulary.workflow_spec_on(sibling).program_revision(),
        application.workflow_spec().program_revision(),
    );
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    let parent_spec = vocabulary.workflow_spec_on(main);
    let early = publish_bound(&application, parent_spec, sibling, "early", 84_100, absent);
    expect_stale_definition_vocabulary(early);
    expect_published(publish(&application, sibling, "unchanged", 84_101));
}

#[test]
fn an_adopted_branch_without_its_program_vocabulary_refuses_workflow_work() {
    let mut application = publish_workflow_on_first_program();
    let main = application.current_world();
    adopt_second_program(&application, main);
    assert_eq!(
        application
            .vocabulary()
            .workflow_spec_on(main)
            .program_revision(),
        application.workflow_spec().program_revision(),
        "a branch whose program has no installed vocabulary falls back to the initial one",
    );
    expect_stale_definition_vocabulary(publish(&application, main, "unserved", 84_150));

    // The refusal claimed nothing: once P1's vocabulary is supported, the
    // same key publishes through it.
    support_workflow_program::<RetentionProgramP1>(&mut application);
    expect_published(publish(&application, main, "unserved", 84_150));
}

#[test]
fn sibling_workflow_lineages_stay_on_their_own_branch() {
    let application = publish_workflow_on_first_program();
    let main = application.current_world();
    let sibling = fork_sibling(&application, main);

    let on_sibling = expect_published(publish(&application, sibling, "completed", 84_200));
    let on_main = expect_published(publish(&application, main, "completed", 84_201));
    // Each branch's current definition is its own publication.
    let replace = |branch, terminal, idempotency, own| {
        let predecessor = WorkflowDefinitionExpectedPredecessor::Published(own);
        let spec = application.vocabulary().workflow_spec_on(branch);
        publish_bound(
            &application,
            spec,
            branch,
            terminal,
            idempotency,
            predecessor,
        )
    };
    expect_published(replace(sibling, "settled", 84_202, on_sibling));
    expect_published(replace(main, "settled", 84_203, on_main));
}

#[test]
fn a_supported_vocabulary_names_one_rostered_program_once() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);

    let duplicate = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .expect("P1 is rostered")
            .installed_program(),
        application.workflow_spec().resources(),
    );
    assert_eq!(
        application.workflow_mut().support_workflow_spec(duplicate),
        Err(WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported),
    );

    let initial = install_workflow_spec(
        application.installed_schema(),
        application.installed_program(),
        application.workflow_spec().resources(),
    );
    assert_eq!(
        application.workflow_mut().support_workflow_spec(initial),
        Err(WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported),
    );

    let other = publish_workflow_on_first_program();
    let foreign_schema = install_workflow_spec(
        other.installed_schema(),
        other.installed_program(),
        application.workflow_spec().resources(),
    );
    assert_eq!(
        application
            .workflow_mut()
            .support_workflow_spec(foreign_schema),
        Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignSchema),
    );
}

#[test]
fn a_retired_program_accepts_no_workflow_vocabulary() {
    let mut application = publish_workflow_on_first_program();
    let late = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .expect("P1 is rostered")
            .installed_program(),
        application.workflow_spec().resources(),
    );
    retire_second_program(&application);
    assert_eq!(
        application.workflow_mut().support_workflow_spec(late),
        Err(WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram),
    );
}

fn second_revision(
    application: &DocumentWorkflowRuntime,
) -> &worth_query_host::facade::declaration::application_program::ApplicationProgramRevision {
    application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
}

fn retire_second_program(application: &DocumentWorkflowRuntime) {
    let revision = *application
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision();
    application
        .retire_program_support(&revision)
        .expect("an idle rostered program retires");
}

fn fork_sibling(
    application: &DocumentWorkflowRuntime,
    parent: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    application
        .runtime()
        .branches()
        .fork(parent)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling branch publishes")
}

fn adopt_second_program(application: &DocumentWorkflowRuntime, branch: WorthQueryProductBranch) {
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
        .prepare(64)
        .expect("an idle workflow host adopts P1");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

/// Publishes a definition bound to the vocabulary `branch` currently runs.
fn publish(
    application: &DocumentWorkflowRuntime,
    branch: WorthQueryProductBranch,
    terminal: &str,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let spec = application.vocabulary().workflow_spec_on(branch);
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    publish_bound(application, spec, branch, terminal, idempotency, absent)
}

/// Publishes a definition bound to `spec` on `branch`, whichever program the
/// branch runs.
fn publish_bound(
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

fn expect_published(
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

fn expect_stale_definition_vocabulary(
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
