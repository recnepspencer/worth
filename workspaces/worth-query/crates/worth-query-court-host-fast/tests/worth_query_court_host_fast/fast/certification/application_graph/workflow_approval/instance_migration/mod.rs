pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_approval::instance_migration::*;

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
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
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

    support_workflow_program::<RetentionProgramP1>(&mut application);
    let cancel = worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions::new()
        .definition(
            &next,
            worth_query_host::facade::primary_graph::WorthQueryWorkflowDefinitionDisposition::Carry,
        )
        .instance(&successor, WorthQueryWorkflowInstanceDisposition::Cancel);
    let cancellation = worth_query_worlds::facade::certification::document_retention_model::workflow::prepare_second_program_adoption(
        &application,
        instance.branch(),
        Some(cancel),
    );
    assert!(matches!(&cancellation,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::WorkflowDispositionRejected))),
        "actual inherited performed effect forbids cancellation at the native safety owner; actual native refusal: {:?}", cancellation.as_ref().err());

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
    assert_eq!(read_retention(application.runtime(), instance.branch()), 8);
}
