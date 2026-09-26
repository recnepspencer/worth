//! A fork holds the definitions it copied under their own identity. Named on
//! the fork, a copied definition governs the fork's new starts, successors
//! and retirement like one the fork published, and none of it reaches the
//! branch it was copied from.

use worth_query_host::facade::{
    application_entry::{WorkflowDefinitionPreparationDenial, WorkflowInstancePreparationDenial},
    primary_graph::WorthQueryApplicationAttemptDenialKind,
    product::WorthQueryProductBranch,
};

use super::*;

#[test]
fn a_fork_decides_the_definition_it_copied_on_its_own_truth() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let first = expect_published(
        "main revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            451,
        ),
    );
    let fork = fork_of(&application, main);
    let copied = first.held_on(fork);
    assert_eq!(copied.entity_id(), first.entity_id());
    assert_eq!(copied.branch(), fork);

    let pinned = expect_started(start_instance(&application, copied.clone(), 452));
    assert_eq!(pinned.branch(), fork);
    let successor = expect_published(
        "fork successor",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(copied.clone()),
            453,
        ),
    );
    assert_eq!(successor.branch(), fork);
    expect_stale_start(start_instance(&application, copied, 454));
    let untouched = expect_started(start_instance(&application, first.clone(), 455));
    assert_eq!(untouched.branch(), main, "main still holds its definition");
    let settled = expect_started(start_instance(&application, successor.clone(), 456));
    assert_eq!(settled.start_node_path(), "settled");

    expect_retired(retire_definition(&application, successor.clone(), 457));
    expect_stale_start(start_instance(&application, successor, 458));
    expect_started(start_instance(&application, first, 459));
    match advance_instance(&application, pinned, 460)
        .expect("pinned fork work prepares against its retained definition")
    {
        WorkflowProgressOutcome::Completed(transition) => {
            assert_eq!(transition.node_path(), "completed");
        }
        other => panic!("the fork's pinned instance completes: {other:?}"),
    }
}

#[test]
fn a_definition_published_on_a_sibling_is_never_another_forks() {
    let application = publish_workflow_on_first_program();
    let main = application.runtime().current_world();
    let first = expect_published(
        "main revision",
        publish_definition(
            &application,
            terminal_definition("completed"),
            WorkflowDefinitionExpectedPredecessor::Absent,
            471,
        ),
    );
    let sibling = fork_of(&application, main);
    let other = fork_of(&application, main);
    let sibling_successor = expect_published(
        "sibling successor",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first.held_on(sibling)),
            472,
        ),
    );
    let own_successor = expect_published(
        "the other fork's successor, with the sibling's exact content",
        publish_definition(
            &application,
            terminal_definition("settled"),
            WorkflowDefinitionExpectedPredecessor::Published(first.held_on(other)),
            477,
        ),
    );
    assert_ne!(own_successor.entity_id(), sibling_successor.entity_id());
    let named_elsewhere = sibling_successor.held_on(other);

    // The other fork holds no such definition to compile or retire, even
    // though it holds one with the same content under its own identity.
    match start_instance(&application, named_elsewhere.clone(), 473) {
        Err(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable
        ),
        other => panic!("the other fork never starts the sibling's definition: {other:?}"),
    }
    expect_stale_publication(publish_definition(
        &application,
        terminal_definition("closed"),
        WorkflowDefinitionExpectedPredecessor::Published(named_elsewhere.clone()),
        474,
    ));
    match retire_definition(&application, named_elsewhere, 475) {
        Err(WorthQueryWorkflowDefinitionRetirementPreparationDenial::DefinitionPreparation(
            WorkflowDefinitionPreparationDenial::Attempt(attempt),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionAffinityMismatch
        ),
        other => panic!("the other fork never retires the sibling's definition: {other:?}"),
    }
    expect_stale_start(start_instance(&application, first.held_on(other), 476));
    let held = expect_started(start_instance(&application, own_successor, 478));
    assert_eq!(
        held.branch(),
        other,
        "the other fork starts its own successor"
    );
    assert_eq!(held.start_node_path(), "settled");
}

fn fork_of(
    application: &BoundedDimensionWorkflowRuntime,
    branch: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    application
        .runtime()
        .branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the fork publishes")
}
