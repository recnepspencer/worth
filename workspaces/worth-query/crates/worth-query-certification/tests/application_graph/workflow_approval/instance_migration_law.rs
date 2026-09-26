//! What a migration refuses: another workflow's definition, a resume that
//! could run before its producer, and every request for a migrated source
//! except the exact replay of its migration.

use worth_query_host::facade::application_entry::{
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};

use super::super::bounded_dimension_model::workflow::{
    migrate_instance, proposal_terminal_definition, reproposing_geometry_definition,
};
use super::instance_migration::{migration_denial, replace_definition, started, supersede};
use super::*;

#[test]
fn a_migrated_source_admits_nothing_but_the_replay_of_its_migration() {
    let (application, definition, instance, proposal, required, _) =
        approval_journey("applied", 87_200);
    let other = match publish_definition(
        &application,
        proposal_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        87_210,
    )
    .expect("another workflow's definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("another workflow's definition did not publish: {other:?}"),
    };
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            instance.clone(),
            other,
            "proposal",
            87_211,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "an instance never migrates to another workflow",
    );
    let superseded = supersede(&application, definition, "done", 87_212);
    let target = supersede(&application, superseded.clone(), "finished", 87_218);
    match migrate_instance(
        &application,
        instance.clone(),
        superseded,
        "propose",
        87_219,
    )
    .expect("a superseded target prepares against the definition it names")
    {
        WorkflowInstanceStartOutcome::Application(WorthQueryApplicationCommitOutcome::Stale(
            stale,
        )) => assert!(stale.stale_fact_count() > 0),
        other => panic!("a superseded definition is never a migration target: {other:?}"),
    }
    let successor = started(migrate_instance(
        &application,
        instance.clone(),
        target.clone(),
        "propose",
        87_213,
    ));
    assert!(!successor.replayed());
    for (resume_at, key) in [("propose", 87_214), ("finished", 87_215)] {
        assert_eq!(
            migration_denial(migrate_instance(
                &application,
                instance.clone(),
                target.clone(),
                resume_at,
                key,
            )),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated,
            "a source migrates once",
        );
    }
    let replay = started(migrate_instance(
        &application,
        instance.clone(),
        target,
        "propose",
        87_213,
    ));
    assert!(replay.replayed());
    assert_eq!(replay.instance(), successor.instance());

    let decision = approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        87_216,
    )
    .expect_err("a migrated instance takes no decision");
    assert!(
        matches!(
            &decision,
            WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(attempt)
            ) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated
        ),
        "{decision:?}",
    );
    let proposal = propose_instance(&application, instance, 87_217)
        .expect_err("a migrated instance takes no proposal");
    assert!(
        matches!(
            &proposal,
            WorthQueryWorkflowProposalPreparationDenial::ProposalPreparation(
                WorkflowProposalPreparationDenial::Attempt(attempt)
            ) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated
        ),
        "{proposal:?}",
    );
}

#[test]
fn a_loop_back_to_a_producer_never_admits_a_resume_before_it() {
    let (application, definition, instance, _, _, _) = approval_journey("applied", 87_300);
    let target = replace_definition(
        &application,
        definition,
        reproposing_geometry_definition("applied"),
        87_310,
    );
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            instance.clone(),
            target.clone(),
            "checks/structural",
            87_311,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "a rejection loop reaches the proposal only after the assessment consumes it",
    );
    let successor = started(migrate_instance(
        &application,
        instance,
        target,
        "propose",
        87_312,
    ));
    assert_eq!(successor.instance().start_node_path(), "propose");
}
