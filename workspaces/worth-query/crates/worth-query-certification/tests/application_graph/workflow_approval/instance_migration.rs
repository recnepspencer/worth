//! Explicit migration continues a live instance on a newer definition as a
//! successor. It carries performed effects as history the successor can never
//! repeat, and re-establishes every proposal, evidence and approval it needs.

use worth_query_host::facade::application_entry::{
    WorkflowInstancePreparationDenial, WorthQueryWorkflowInstanceStartPreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryWorkflowInstanceCustody, WorthQueryWorkflowInstanceDisposition,
};

use super::super::bounded_dimension_model::{
    programs::DimensionProgramP1,
    workflow::{migrate_instance, second_program_workflow_inventory, support_workflow_program},
};
use super::*;

#[test]
fn a_waiting_instance_continues_on_the_new_definition_and_its_source_ends() {
    let (application, definition, instance, _, _, _) = approval_journey("applied", 87_000);
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            instance.clone(),
            definition.clone(),
            "propose",
            87_009,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "an instance migrates only to another definition of its workflow",
    );
    let target = supersede(&application, definition.clone(), "done", 87_010);
    for (resume_at, reason) in [
        ("nowhere", "the resume node must be in the target"),
        (
            "approval",
            "an approval cannot resume without its proposal and evidence",
        ),
    ] {
        assert_eq!(
            migration_denial(migrate_instance(
                &application,
                instance.clone(),
                target.clone(),
                resume_at,
                87_011,
            )),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
            "{reason}",
        );
    }

    let successor = started(migrate_instance(
        &application,
        instance.clone(),
        target.clone(),
        "propose",
        87_013,
    ));
    assert!(!successor.replayed());
    let successor = successor.instance().clone();
    assert_ne!(successor.entity_id(), instance.entity_id());
    assert_eq!(successor.definition_entity_id(), target.entity_id());
    assert_eq!(successor.start_node_path(), "propose");
    let replay = started(migrate_instance(
        &application,
        instance.clone(),
        target,
        "propose",
        87_013,
    ));
    assert!(replay.replayed());
    assert_eq!(replay.instance(), &successor);

    match advance_instance(&application, instance.clone(), 87_014) {
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrated
        ),
        other => panic!("a migrated instance names its migration: {other:?}"),
    }
    super::proposal::published_proposal(&application, successor.clone(), 87_015);
    for key in [87_016, 87_018] {
        let settlement = settle_assessment(&application, successor.clone(), key);
        assert!(matches!(
            accept_assessment(&application, successor.clone(), &settlement, key + 1),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert!(matches!(
        advance_instance(&application, successor.clone(), 87_020),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    assert!(matches!(
        advance_instance(&application, successor, 87_021),
        Ok(WorkflowProgressOutcome::AwaitingApproval(_))
    ));
}

#[test]
fn a_performed_effect_carries_as_history_the_successor_never_repeats() {
    let (mut application, definition, instance, proposal, required, _) =
        approval_journey("applied", 87_100);
    match approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        87_110,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected approval to complete, got {other:?}"),
    }
    let target = supersede(&application, definition, "done", 87_111);
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            instance.clone(),
            target.clone(),
            "done",
            87_112,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled,
        "an approved operation settles under the source before migration",
    );

    perform_approved_effect(&application, &instance, 87_113);
    assert_eq!(read_dimension(application.runtime(), instance.branch()), 8);
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            instance.clone(),
            target.clone(),
            "propose",
            87_116,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "a successor must not be able to run a performed effect again",
    );
    let first = started(migrate_instance(
        &application,
        instance.clone(),
        target.clone(),
        "done",
        87_117,
    ))
    .instance()
    .clone();
    // A second hop carries what the first successor inherited.
    let next = supersede(&application, target, "finished", 87_119);
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            first.clone(),
            next.clone(),
            "propose",
            87_120,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "an inherited effect is never run again either",
    );
    let successor = started(migrate_instance(
        &application,
        first.clone(),
        next.clone(),
        "finished",
        87_121,
    ))
    .instance()
    .clone();

    support_workflow_program::<DimensionProgramP1>(&mut application);
    let inventory = second_program_workflow_inventory(&application, instance.branch());
    assert!(inventory.instance(instance.entity_id()).is_none());
    assert!(inventory.instance(first.entity_id()).is_none());
    let carried = inventory
        .instance(successor.entity_id())
        .expect("the successor is the live instance");
    assert_eq!(
        carried.custody(),
        &WorthQueryWorkflowInstanceCustody::Performed
    );
    assert_eq!(
        carried.legal_dispositions(),
        &[WorthQueryWorkflowInstanceDisposition::Carry],
        "inherited effects forbid cancelling the successor",
    );

    match advance_instance(&application, successor.clone(), 87_124) {
        Ok(WorkflowProgressOutcome::Completed(performed)) => {
            assert!(performed.terminal());
            assert_eq!(performed.node_path(), "finished");
        }
        other => panic!("the successor completes at its resume terminal: {other:?}"),
    }
    let last = supersede(&application, next, "closed", 87_122);
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            successor,
            last,
            "closed",
            87_123,
        )),
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        "a completed instance has no work left to migrate",
    );
    assert_eq!(read_dimension(application.runtime(), instance.branch()), 8);
}

pub(super) fn supersede(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    completion: &str,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    replace_definition(
        application,
        definition,
        reviewed_geometry_definition(completion),
        key,
    )
}

pub(super) fn replace_definition(
    application: &BoundedDimensionWorkflowRuntime,
    definition: PublishedWorkflowDefinitionRef,
    replacement: worth_query_host::facade::declaration::application_program::ValidatedWorkflowDefinition<
        super::super::bounded_dimension_model::workflow::ReviewedGeometryWorkflow,
    >,
    key: u64,
) -> PublishedWorkflowDefinitionRef {
    match publish_definition(
        application,
        replacement,
        WorkflowDefinitionExpectedPredecessor::Published(definition),
        key,
    )
    .expect("the successor definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the successor definition did not publish: {other:?}"),
    }
}

pub(super) fn perform_approved_effect(
    application: &BoundedDimensionWorkflowRuntime,
    instance: &PublishedWorkflowInstanceRef,
    key: u64,
) {
    let required = match advance_instance(application, instance.clone(), key)
        .expect("the approved operation requirement prepares")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&(key + 1))
        .for_workflow_operation(application, &required)
        .expect("the effect request matches the operation requirement")
        .execute_in_program(application.program_runtime())
        .expect("the approved effect executes");
    assert!(matches!(
        effect,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

pub(super) fn started(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) -> worth_query_host::facade::application_entry::PerformedWorkflowInstanceStart {
    match outcome.expect("the migration prepares") {
        WorkflowInstanceStartOutcome::Started(performed) => performed,
        other => panic!("the migration did not publish: {other:?}"),
    }
}

pub(super) fn migration_denial(
    outcome: Result<WorkflowInstanceStartOutcome, WorthQueryWorkflowInstanceStartPreparationDenial>,
) -> WorthQueryApplicationAttemptDenialKind {
    match outcome {
        Err(WorthQueryWorkflowInstanceStartPreparationDenial::InstancePreparation(
            WorkflowInstancePreparationDenial::Attempt(attempt),
        )) => attempt.kind(),
        other => panic!("expected a typed migration denial, got {other:?}"),
    }
}
