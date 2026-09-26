//! A fork copies an instance's history, never its execution. Work continues
//! on the fork only as an explicitly admitted successor under the migration
//! law, and the instance on its own branch is untouched.

use worth_query_host::facade::primary_graph::{
    WorthQueryWorkflowInstanceCustody, WorthQueryWorkflowInstanceDisposition,
};
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::super::bounded_dimension_model::{
    programs::DimensionProgramP1,
    workflow::{continue_on_fork, second_program_workflow_inventory, support_workflow_program},
};
use super::instance_migration::{migration_denial, perform_approved_effect, started, supersede};
use super::journey::approval_requirement;
use super::*;

#[test]
fn a_fork_continues_a_waiting_instance_only_as_a_new_admitted_instance() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("applied", 88_000);
    let fork = fork_of(&application, instance.branch());
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            instance.branch(),
            instance.clone(),
            definition.clone(),
            "propose",
            88_010,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
        "an instance continues on its own branch only by migration",
    );
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            fork,
            instance.clone(),
            definition.clone(),
            "approval",
            88_011,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "a copied proposal and its evidence never open the approval",
    );

    let successor = started(continue_on_fork(
        &application,
        fork,
        instance.clone(),
        definition.clone(),
        "propose",
        88_012,
    ));
    assert!(!successor.replayed());
    let successor = successor.instance().clone();
    assert_eq!(successor.branch(), fork);
    assert_ne!(successor.entity_id(), instance.entity_id());
    assert_eq!(successor.definition_entity_id(), definition.entity_id());
    assert_eq!(successor.start_node_path(), "propose");
    let replay = started(continue_on_fork(
        &application,
        fork,
        instance.clone(),
        definition.clone(),
        "propose",
        88_012,
    ));
    assert!(replay.replayed());
    assert_eq!(replay.instance(), &successor);
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            fork,
            instance.clone(),
            definition,
            "propose",
            88_013,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated,
        "a fork continues its copy once",
    );

    let (fork_proposal, fork_required, _) =
        approval_requirement(&application, successor.clone(), 88_020);
    for (replay, attempt) in [(false, "decides"), (true, "replays")] {
        match approve_instance(
            &application,
            successor.clone(),
            &fork_required,
            &fork_proposal,
            WorkflowApprovalDecision::Approve,
            88_030,
        ) {
            Ok(WorkflowProgressOutcome::Completed(performed)) => assert_eq!(
                performed.replayed(),
                replay,
                "the successor {attempt} its decision against the fork's root",
            ),
            other => panic!("the successor {attempt} its own decision on the fork: {other:?}"),
        }
    }
    perform_approved_effect(&application, &successor, 88_031);
    assert_eq!(read_dimension(application.runtime(), fork), 8);
    assert!(
        matches!(
            approve_instance(
                &application,
                instance.clone(),
                &required,
                &proposal,
                WorkflowApprovalDecision::Approve,
                88_015,
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ),
        "the instance on its own branch still takes its decision",
    );
    assert_eq!(
        read_dimension(application.runtime(), instance.branch()),
        SEED_DIMENSION,
        "the fork's effect never lands on the source branch",
    );
}

#[test]
fn a_fork_carries_performed_effects_and_never_continues_an_unsettled_approval() {
    let (mut application, definition, instance, proposal, required, _) =
        approval_journey("applied", 88_100);
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        88_110,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("the approval did not complete: {other:?}"),
    }
    let target = supersede(&application, definition.clone(), "done", 88_111);
    let early = fork_of(&application, instance.branch());
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            early,
            instance.clone(),
            target.clone(),
            "done",
            88_112,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled,
        "an approved operation settles on its own branch before a fork continues it",
    );

    perform_approved_effect(&application, &instance, 88_113);
    let fork = fork_of(&application, instance.branch());
    assert_eq!(read_dimension(application.runtime(), fork), 8);
    match continue_on_fork(
        &application,
        fork,
        instance.clone(),
        definition,
        "applied",
        88_119,
    ) {
        Ok(WorkflowInstanceStartOutcome::Application(
            WorthQueryApplicationCommitOutcome::Stale(_),
        )) => {}
        other => panic!("a fork continues only under a definition it holds current: {other:?}"),
    }
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            fork,
            instance.clone(),
            target.clone(),
            "propose",
            88_115,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "a copied receipt never lets the fork repeat its effect",
    );
    let successor = started(continue_on_fork(
        &application,
        fork,
        instance.clone(),
        target.clone(),
        "done",
        88_116,
    ))
    .instance()
    .clone();

    let later = supersede(&application, target, "finished", 88_117);
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            early,
            instance.clone(),
            later,
            "finished",
            88_118,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowDefinitionCompilationUnavailable,
        "a definition published after the fork is not the fork's to continue under",
    );

    support_workflow_program::<DimensionProgramP1>(&mut application);
    assert!(
        second_program_workflow_inventory(&application, early)
            .instances()
            .is_empty(),
        "a copied instance is historical on the fork until a continuation",
    );
    let forked = second_program_workflow_inventory(&application, fork);
    assert_eq!(forked.instances().len(), 1);
    assert!(forked.instance(instance.entity_id()).is_none());
    let carried = forked
        .instance(successor.entity_id())
        .expect("the successor is the fork's live instance");
    assert_eq!(
        carried.custody(),
        &WorthQueryWorkflowInstanceCustody::Performed
    );
    assert_eq!(
        carried.legal_dispositions(),
        &[WorthQueryWorkflowInstanceDisposition::Carry],
    );
    assert!(
        second_program_workflow_inventory(&application, instance.branch())
            .instance(instance.entity_id())
            .is_some(),
        "the instance still needs its disposition on its own branch",
    );
    assert_eq!(read_dimension(application.runtime(), instance.branch()), 8);
    assert_eq!(read_dimension(application.runtime(), fork), 8);
}

#[test]
fn a_fork_never_continues_a_siblings_copy() {
    let (application, definition, instance, _, _, _) = approval_journey("applied", 88_200);
    let main = instance.branch();
    let sibling = fork_of(&application, main);
    let fork = fork_of(&application, main);
    let on_sibling = started(continue_on_fork(
        &application,
        sibling,
        instance,
        definition.clone(),
        "propose",
        88_210,
    ))
    .instance()
    .clone();
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            fork,
            on_sibling.clone(),
            definition.clone(),
            "propose",
            88_211,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAffinityMismatch,
        "a sibling's instance and its definition are neither the fork's nor its copy's",
    );
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            fork,
            on_sibling,
            definition.held_on(fork),
            "propose",
            88_212,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        "fork truth never holds a sibling's copy, even under the fork's own definition",
    );
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
