use super::support::*;

#[test]
fn a_durable_open_without_its_source_lease_fails_closed_as_incomplete_history() {
    let directory = tempfile::tempdir().expect("temp directory");
    let control = OperationalControlStore::open(
        OperationalControlLocation::new(
            directory.path().join("control.log"),
            failure_domain("control-media"),
        ),
        std::iter::empty::<ProtectedOperationalMediaLocation>(),
    )
    .expect("control store");
    let authority = crate::backup::export::current_authority("s10-incomplete-workflow-open");
    let operation = OperationalOperationId::new("incomplete-workflow-open").expect("operation");
    control
        .append(&OperationalControlRecord::workflow_opened(
            authority.authority_identity(),
            operation.clone(),
            OperationalTransitionId::new("incomplete-workflow-open:opened").expect("transition"),
            OperationalWorkflowKind::Backup,
        ))
        .expect("legacy workflow open");

    assert_invalid_history(
        &control,
        &authority,
        ControlStoreGeneration::from_raw(1).expect("opened generation"),
        &operation,
        crate::OperationalControlHistoryViolationKind::WorkflowOpenWithoutDurableSourceLease,
    );
}

#[test]
fn owner_release_cannot_bypass_materialization_and_independent_verification() {
    let scenario = BackupScenario::new("verification-before-materialization");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let operation =
        OperationalOperationId::new("verification-before-materialization").expect("operation");
    let admitted = OnlineBackupIntent::new(
        operation.clone(),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("active cut");
    control
        .append(&OperationalControlRecord::from_persisted_parts(
            authority.authority_identity(),
            operation.clone(),
            OperationalTransitionId::new("verification-before-materialization:forged-terminal")
                .expect("transition"),
            crate::OperationalControlRecordKind::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
                verification_identity: [0x61; 32],
                release: admitted.cut().lease().release_record(),
            },
        ))
        .expect("byte-valid but out-of-order terminal record");

    assert_invalid_history(
        &control,
        &authority,
        ControlStoreGeneration::from_raw(2).expect("terminal generation"),
        &operation,
        crate::OperationalControlHistoryViolationKind::VerificationBeforeMaterialization,
    );
}

#[test]
fn terminal_release_must_name_the_operations_exact_cut() {
    let scenario = BackupScenario::new("wrong-cut-terminal-release");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let operation = OperationalOperationId::new("wrong-cut-terminal-release").expect("operation");
    let _admitted = OnlineBackupIntent::new(
        operation.clone(),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("active cut");
    control
        .append(&OperationalControlRecord::from_persisted_parts(
            authority.authority_identity(),
            operation.clone(),
            OperationalTransitionId::new("wrong-cut-terminal-release:forged-terminal")
                .expect("transition"),
            crate::OperationalControlRecordKind::BackupAbandoned {
                reason: "wrong cut".into(),
                released_source_lease: release([0xee; 32]),
            },
        ))
        .expect("byte-valid wrong-cut release");

    assert_invalid_history(
        &control,
        &authority,
        ControlStoreGeneration::from_raw(2).expect("terminal generation"),
        &operation,
        crate::OperationalControlHistoryViolationKind::TerminalReleaseCutMismatch,
    );
}

#[test]
fn backup_records_cannot_attach_to_another_workflow_kind() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("control.log");
    let control = OperationalControlStore::open(
        OperationalControlLocation::new(&path, failure_domain("control-media")),
        std::iter::empty::<ProtectedOperationalMediaLocation>(),
    )
    .expect("control store");
    let authority = crate::backup::export::current_authority("s10-wrong-workflow-record");
    let operation = OperationalOperationId::new("wrong-workflow-record").expect("operation");
    control
        .append(&OperationalControlRecord::workflow_opened(
            authority.authority_identity(),
            operation.clone(),
            OperationalTransitionId::new("wrong-workflow-record:opened").expect("transition"),
            OperationalWorkflowKind::OfflineInspection,
        ))
        .expect("workflow open");
    control
        .append(&OperationalControlRecord::from_persisted_parts(
            authority.authority_identity(),
            operation.clone(),
            OperationalTransitionId::new("wrong-workflow-record:materialized").expect("transition"),
            crate::OperationalControlRecordKind::BackupMaterializationRecorded {
                manifest_digest: [0x71; 32],
            },
        ))
        .expect("byte-valid cross-workflow record");

    assert_invalid_history(
        &control,
        &authority,
        ControlStoreGeneration::from_raw(2).expect("second generation"),
        &operation,
        crate::OperationalControlHistoryViolationKind::BackupRecordForDifferentWorkflow {
            workflow: OperationalWorkflowKind::OfflineInspection,
        },
    );
}

#[test]
fn no_record_can_reopen_a_terminal_backup_lifecycle() {
    let scenario = BackupScenario::new("record-after-terminal");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let operation = OperationalOperationId::new("record-after-terminal").expect("operation");
    OnlineBackupIntent::new(
        operation.clone(),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("active cut")
    .abandon("terminal", &control, &scenario.leases)
    .expect("durable terminal");
    control
        .append(&OperationalControlRecord::from_persisted_parts(
            authority.authority_identity(),
            operation.clone(),
            OperationalTransitionId::new("record-after-terminal:materialized").expect("transition"),
            crate::OperationalControlRecordKind::BackupMaterializationRecorded {
                manifest_digest: [0x81; 32],
            },
        ))
        .expect("byte-valid record after terminal");

    assert_invalid_history(
        &control,
        &authority,
        ControlStoreGeneration::from_raw(3).expect("post-terminal generation"),
        &operation,
        crate::OperationalControlHistoryViolationKind::RecordAfterTerminal,
    );
}

#[test]
fn archived_workflow_semantics_remain_exact_after_the_replay_index_spills_to_disk() {
    let directory = tempfile::tempdir().expect("temp directory");
    let control = OperationalControlStore::open(
        OperationalControlLocation::new(
            directory.path().join("control.log"),
            failure_domain("control-media"),
        ),
        std::iter::empty::<ProtectedOperationalMediaLocation>(),
    )
    .expect("control store");
    let authority = crate::backup::export::current_authority("s10-archived-workflow-spill");
    for index in 0..1_100 {
        control
            .append(&OperationalControlRecord::workflow_opened(
                authority.authority_identity(),
                OperationalOperationId::new(format!("archived-workflow-{index}"))
                    .expect("operation"),
                OperationalTransitionId::new(format!("archived-workflow-{index}:opened"))
                    .expect("transition"),
                OperationalWorkflowKind::OfflineInspection,
            ))
            .expect("archived workflow");
    }
    let duplicate = OperationalOperationId::new("archived-workflow-0").expect("duplicate");
    control
        .append(&OperationalControlRecord::workflow_opened(
            authority.authority_identity(),
            duplicate.clone(),
            OperationalTransitionId::new("archived-workflow-0:opened-again").expect("transition"),
            OperationalWorkflowKind::OfflineInspection,
        ))
        .expect("byte-valid duplicate workflow open");

    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(1_101).expect("selected generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    assert!(matches!(
        control.inspect_generations(&fencing),
        ControlStoreTrustPosture::Indeterminate(
            crate::ControlStoreSelectionIndeterminate::InvalidHistory(violation)
        ) if violation.record_index() == 1_100
            && violation.operation_id() == &duplicate
            && violation.kind()
                == &crate::OperationalControlHistoryViolationKind::DuplicateWorkflowOpen
    ));
}

fn assert_invalid_history(
    control: &OperationalControlStore,
    authority: &worth_store_authority::StoreCurrentAuthorityWitness,
    generation: ControlStoreGeneration,
    operation: &OperationalOperationId,
    expected: crate::OperationalControlHistoryViolationKind,
) {
    let selection = TestControlStoreFencingProvider::selected(authority, control, generation);
    let fencing = ControlStoreFencingAuthority::for_current_store(authority, &selection);
    assert!(matches!(
        control.inspect_generations(&fencing),
        ControlStoreTrustPosture::Indeterminate(
            crate::ControlStoreSelectionIndeterminate::InvalidHistory(violation)
        ) if violation.operation_id() == operation && violation.kind() == &expected
    ));
}

fn release(
    cut_identity: [u8; 32],
) -> worth_store_physical_isolation::BackupReachabilityLeaseReleaseRecord {
    let mut encoded = [0; 36];
    encoded[..4].copy_from_slice(b"WBR1");
    encoded[4..].copy_from_slice(&cut_identity);
    worth_store_physical_isolation::BackupReachabilityLeaseReleaseRecord::recover(&encoded)
        .expect("canonical release record")
}
