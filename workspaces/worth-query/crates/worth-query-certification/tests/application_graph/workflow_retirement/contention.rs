//! Retirement and reopening race other definition authors between prepare
//! and commit. An interleaved commit moves the product basis, so the loser is
//! denied before effects; re-preparing observes the winner's currentness.

use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind;

use super::*;

#[test]
fn a_successor_published_after_prepare_makes_the_retirement_stale() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            501,
        ),
    );
    let mut successor = None;
    let outcome = retire_after(&application, first.clone(), 502, || {
        successor = Some(expect_published(
            "competing successor",
            publish_definition(
                &application,
                terminal_definition("settled"),
                WorkflowDefinitionExpectedPredecessor::Published(first.clone()),
                503,
            ),
        ));
    });
    expect_basis_stale(outcome.map(|outcome| match outcome {
        WorkflowDefinitionRetirementOutcome::Application(outcome) => outcome,
        other => panic!("the loser must not retire, got {other:?}"),
    }));
    expect_stale_retirement(retire_definition(&application, first.clone(), 502));
    let successor = successor.expect("the competing publication ran");
    expect_started(start_instance(&application, successor, 504));
    expect_stale_start(start_instance(&application, first, 505));
}

#[test]
fn a_competing_retirement_after_prepare_leaves_one_committed_retirement() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            511,
        ),
    );
    let outcome = retire_after(&application, first.clone(), 512, || {
        let winner = expect_retired(retire_definition(&application, first.clone(), 513));
        assert!(!winner.replayed());
    });
    expect_basis_stale(outcome.map(|outcome| match outcome {
        WorkflowDefinitionRetirementOutcome::Application(outcome) => outcome,
        other => panic!("the loser must not retire, got {other:?}"),
    }));
    expect_stale_retirement(retire_definition(&application, first.clone(), 512));
    expect_stale_start(start_instance(&application, first, 514));
}

#[test]
fn only_one_of_two_prepared_reopens_commits() {
    let application = publish_workflow_on_first_program();
    let first = expect_published(
        "first revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            521,
        ),
    );
    expect_retired(retire_definition(&application, first, 522));
    let mut winner = None;
    let loser = publish_after(&application, terminal_definition("late"), 523, || {
        winner = Some(expect_published(
            "winning reopen",
            publish_definition(
                &application,
                terminal_definition("early"),
                WorkflowDefinitionExpectedPredecessor::Absent,
                524,
            ),
        ));
    });
    expect_basis_stale(loser.map(|outcome| match outcome {
        WorkflowDefinitionPublicationOutcome::Application(outcome) => outcome,
        other => panic!("the losing reopen must not publish, got {other:?}"),
    }));
    expect_stale_publication(publish_definition(
        &application,
        terminal_definition("late"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        523,
    ));
    let winner = winner.expect("the winning reopen ran");
    let started = expect_started(start_instance(&application, winner, 525));
    assert_eq!(started.start_node_path(), "early");
}

fn retire_after(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    idempotency: u64,
    between: impl FnOnce(),
) -> Result<
    WorkflowDefinitionRetirementOutcome,
    WorthQueryWorkflowDefinitionRetirementPreparationDenial,
> {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_definition_retirement(application, definition)?;
    between();
    Ok(prepared.execute())
}

fn publish_after(
    application: &BoundedDimensionWorkflowRuntime,
    definition: ValidatedWorkflowDefinition<ReviewedGeometryWorkflow>,
    idempotency: u64,
    between: impl FnOnce(),
) -> Result<
    WorkflowDefinitionPublicationOutcome,
    WorthQueryWorkflowDefinitionPublicationPreparationDenial,
> {
    let contract = application
        .workflow_spec()
        .bind_definition(definition)
        .expect("the workflow definition binds to installed vocabulary");
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepared = runtime
        .request(&principal, &scope)
        .mutate(authoring_intent())
        .without_source()
        .idempotency(&idempotency)
        .prepare_workflow_publication(contract, WorkflowDefinitionExpectedPredecessor::Absent)?;
    between();
    Ok(prepared.execute())
}

fn expect_basis_stale<Denial: std::fmt::Debug>(
    result: Result<WorthQueryApplicationCommitOutcome, Denial>,
) {
    match result.expect("the losing request prepared before the winner committed") {
        WorthQueryApplicationCommitOutcome::Denied(denial) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProductBasisStale
        ),
        other => panic!("expected the moved product basis to deny, got {other:?}"),
    }
}
