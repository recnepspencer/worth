//! An adopted branch runs workflows through the vocabulary installed against
//! the program it now runs, and only that one.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, WorkflowDefinitionBindingDenial,
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPreparationDenial,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceBindingDenial,
    WorkflowInstancePreparationDenial, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryApplicationRequestExt, WorthQueryWorkflowAdvancePreparationDenial,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
    WorthQueryWorkflowInstanceStartPreparationDenial,
};
use worth_query_host::facade::application_installation::{
    WorthQueryWorkflowRuntimeBindingDenial, WorthQueryWorkflowVocabulary,
};
use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome;
use worth_query_host::facade::product::WorthQueryProductBranch;

use crate::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    programs::{DimensionProgramP0, DimensionProgramP1},
    schema::BoundedDimensionSchema,
    workflow::{
        authoring_intent, install_workflow_spec, support_workflow_program, terminal_definition,
        ReviewedGeometryWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
    },
};

type Vocabulary<'application, Program> = WorthQueryWorkflowVocabulary<
    'application,
    BoundedDimensionSchema,
    ReviewedGeometryWorkflow,
    Program,
>;

#[test]
fn an_adopted_branch_runs_workflows_through_its_new_program_vocabulary() {
    let mut application = publish_workflow_on_first_program();
    assert!(
        application
            .supported_vocabulary::<DimensionProgramP1>()
            .is_none(),
        "a rostered program has no workflow vocabulary until one is installed",
    );
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    adopt_second_program(&application, main);

    let first = application.vocabulary();
    let second = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("the supported vocabulary serves while P1 is rostered");
    assert_ne!(
        first.workflow_spec().program_revision(),
        second.workflow_spec().program_revision(),
    );

    let definition = expect_published(publish(second, main, "completed", 84_000));
    let instance = match start(second, main, definition.clone(), 84_001)
        .expect("the P1 instance prepares on the adopted branch")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the P1 instance did not start: {other:?}"),
    };
    assert_eq!(
        instance.program_revision(),
        second.workflow_spec().program_revision()
    );
    match advance(second, main, instance, 84_002)
        .expect("the P1 advance prepares on the adopted branch")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
            assert!(transition.terminal());
        }
        other => panic!("the P1 instance did not complete: {other:?}"),
    }

    expect_stale_definition_vocabulary(publish(first, main, "stale", 84_003));
    expect_stale_instance_vocabulary(start(first, main, definition, 84_004));
}

#[test]
fn a_sibling_keeps_its_program_vocabulary_after_its_parent_adopts() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<DimensionProgramP1>(&mut application);
    let main = application.current_world();
    let sibling = fork_sibling(&application, main);
    adopt_second_program(&application, main);

    let second = application
        .supported_vocabulary::<DimensionProgramP1>()
        .expect("the supported vocabulary serves while P1 is rostered");
    expect_stale_definition_vocabulary(publish(second, sibling, "early", 84_100));
    expect_published(publish(
        application.vocabulary(),
        sibling,
        "unchanged",
        84_101,
    ));
}

#[test]
fn sibling_workflow_lineages_stay_on_their_own_branch() {
    let application = publish_workflow_on_first_program();
    let vocabulary = application.vocabulary();
    let main = application.current_world();
    let sibling = fork_sibling(&application, main);

    let on_sibling = expect_published(publish(vocabulary, sibling, "completed", 84_200));
    let on_main = expect_published(publish(vocabulary, main, "completed", 84_201));
    // Each branch's current definition is its own publication.
    let replace = |branch, terminal, idempotency, own| {
        let predecessor = WorkflowDefinitionExpectedPredecessor::Published(own);
        publish_after(vocabulary, branch, terminal, idempotency, predecessor)
    };
    expect_published(replace(sibling, "settled", 84_202, on_sibling));
    expect_published(replace(main, "settled", 84_203, on_main));
}

#[test]
fn a_supported_vocabulary_names_one_rostered_program_once() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<DimensionProgramP1>(&mut application);

    let duplicate = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<DimensionProgramP1>()
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
    assert!(application
        .supported_vocabulary::<DimensionProgramP0>()
        .is_none());
}

#[test]
fn a_retired_program_keeps_no_workflow_vocabulary() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<DimensionProgramP1>(&mut application);
    retire_second_program(&application);
    assert!(
        application
            .supported_vocabulary::<DimensionProgramP1>()
            .is_none(),
        "retirement withdraws the vocabulary installed against the program",
    );

    let mut application = publish_workflow_on_first_program();
    let late = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<DimensionProgramP1>()
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

fn retire_second_program(
    application: &crate::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime,
) {
    let revision = application
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
        .clone();
    application
        .retire_program_support(&revision)
        .expect("an idle rostered program retires");
}

fn fork_sibling(
    application: &crate::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime,
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

fn adopt_second_program(
    application: &crate::bounded_dimension_model::host::BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
) {
    let target = application
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
        .clone();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("an idle workflow host adopts P1");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

fn publish<Program>(
    vocabulary: Vocabulary<'_, Program>,
    branch: WorthQueryProductBranch,
    terminal: &str,
    idempotency: u64,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
{
    let absent = WorkflowDefinitionExpectedPredecessor::Absent;
    publish_after(vocabulary, branch, terminal, idempotency, absent)
}

fn publish_after<Program>(
    vocabulary: Vocabulary<'_, Program>,
    branch: WorthQueryProductBranch,
    terminal: &str,
    idempotency: u64,
    predecessor: WorkflowDefinitionExpectedPredecessor,
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
{
    let contract = vocabulary
        .workflow_spec()
        .bind_definition(terminal_definition(terminal))
        .expect("the definition binds to the vocabulary");
    let runtime = vocabulary.runtime();
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

fn start<Program>(
    vocabulary: Vocabulary<'_, Program>,
    branch: WorthQueryProductBranch,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
) -> Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
{
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowInstanceStartIntent {
            input: WorkflowInstanceStartInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_instance_start(vocabulary, definition)
        .map(|request| request.execute())
}

fn advance<Program>(
    vocabulary: Vocabulary<'_, Program>,
    branch: WorthQueryProductBranch,
    instance: PublishedWorkflowInstanceRef,
    idempotency: u64,
) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowAdvancePreparationDenial>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
{
    let runtime = vocabulary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_advance(vocabulary, instance)
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

fn expect_stale_instance_vocabulary(
    result: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) {
    match result {
        Err(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Binding(
                WorkflowInstanceBindingDenial::ProgramRevisionChanged,
            ),
        )) => {}
        other => panic!("another program's vocabulary must not start: {other:?}"),
    }
}
