use super::support::*;

#[test]
fn admitted_cut_reopens_idempotently_and_abandonment_durably_releases_its_lease() {
    let scenario = BackupScenario::new("cut-reopen");
    let authority = scenario.authority();
    let operation = OperationalOperationId::new("backup-cut-reopen").expect("operation");
    let first_control = scenario.control_store();
    let first = OnlineBackupIntent::new(
        operation.clone(),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &first_control, &scenario.leases)
    .expect("first admission");
    let cut_identity = first.cut().identity();
    drop(first);
    drop(first_control);

    let reopened_control = scenario.control_store();
    let resumed = OnlineBackupIntent::new(
        operation,
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &reopened_control, &scenario.leases)
    .expect("idempotent resume");
    assert_eq!(resumed.cut().identity(), cut_identity);
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("idempotent holder registry")
            .active_holders(),
        1,
        "replaying one operation must not create a second lease holder"
    );
    resumed
        .abandon("operator cancellation", &reopened_control, &scenario.leases)
        .expect("durable abandonment");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease registry")
            .active_leases(),
        0
    );

    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &reopened_control,
        ControlStoreGeneration::from_raw(2).expect("generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    match reopened_control.inspect_generations(&fencing) {
        ControlStoreTrustPosture::Selected(selected) => {
            assert_eq!(selected.history_summary().abandoned_backups(), 1);
            assert!(selected.active_backup_recovery_handles().is_empty());
        }
        posture => panic!("abandonment state should be selected: {posture:?}"),
    }
}

#[test]
fn concurrent_backups_of_one_cut_keep_reclaim_protection_until_the_last_release() {
    let scenario = BackupScenario::new("shared-cut-lease-holders");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let first = OnlineBackupIntent::new(
        OperationalOperationId::new("shared-cut-first").expect("first operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("first durable cut holder");
    let second = OnlineBackupIntent::new(
        OperationalOperationId::new("shared-cut-second").expect("second operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("second durable cut holder");
    assert_eq!(first.cut().identity(), second.cut().identity());
    let live = scenario
        .leases
        .live_index_snapshot()
        .expect("live shared-cut registry");
    assert_eq!(live.active_leases(), 1);
    assert_eq!(live.active_holders(), 2);

    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(2).expect("two admitted operations"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let one_workflow_budget = OperationalControlReplayBudget::new(1, 64 * 1024, 64 * 1024)
        .expect("bounded replay budget");
    assert!(matches!(
        control.inspect_generations_with_budget(&fencing, one_workflow_budget),
        ControlStoreTrustPosture::Unavailable(
            ControlStoreAvailabilityDenial::ReplayBudgetExceeded {
                resource: OperationalControlReplayResource::ActiveWorkflows,
                required: 2,
                limit: 1,
            }
        )
    ));
    let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing) else {
        panic!("two same-cut operations must replay as independent holders");
    };
    assert_eq!(selected.active_backup_recovery_handles().len(), 2);
    let recovered_leases = selected
        .recover_backup_reachability_leases()
        .expect("fresh-process holder reconstruction");
    assert_eq!(
        recovered_leases
            .live_index_snapshot()
            .expect("recovered shared-cut registry")
            .active_holders(),
        2
    );

    first
        .abandon("first operation finished", &control, &recovered_leases)
        .expect("first durable release");
    let after_first = recovered_leases
        .live_index_snapshot()
        .expect("one holder remains");
    assert_eq!(after_first.active_leases(), 1);
    assert_eq!(after_first.active_holders(), 1);

    second
        .abandon("second operation finished", &control, &recovered_leases)
        .expect("last durable release");
    let after_last = recovered_leases
        .live_index_snapshot()
        .expect("all holders released");
    assert_eq!(after_last.active_leases(), 0);
    assert_eq!(after_last.active_holders(), 0);
}

#[test]
fn control_replay_rejects_a_recovery_object_before_exceeding_its_byte_budget() {
    let scenario = BackupScenario::new("bounded-control-recovery-object");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let _admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("bounded-control-recovery-object").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(1).expect("lease generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let tiny_object_budget =
        OperationalControlReplayBudget::new(1, 1, 1).expect("one-byte object budget");

    assert!(matches!(
        control.inspect_generations_with_budget(&fencing, tiny_object_budget),
        ControlStoreTrustPosture::Unavailable(
            ControlStoreAvailabilityDenial::ReplayBudgetExceeded {
                resource: OperationalControlReplayResource::SingleRecoveryObjectBytes,
                required,
                limit: 1,
            }
        ) if required > 1
    ));
}

#[test]
fn fresh_process_recovers_the_physical_reclaim_interlock_from_control_media() {
    const CHILD_CONTROL: &str = "WORTH_STORE_S10_LEASE_CONTROL";
    if let Some(control_path) = std::env::var_os(CHILD_CONTROL) {
        let control = OperationalControlStore::open(
            OperationalControlLocation::new(control_path, failure_domain("control-media")),
            std::iter::empty::<ProtectedOperationalMediaLocation>(),
        )
        .expect("child reopens control media");
        let authority = crate::backup::export::current_authority("store.physical.default_instance");
        let selection = TestControlStoreFencingProvider::selected(
            &authority,
            &control,
            ControlStoreGeneration::from_raw(1).expect("lease generation"),
        );
        let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
        let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing)
        else {
            panic!("child must select the durable lease generation");
        };
        let leases = selected
            .recover_backup_reachability_leases()
            .expect("child reconstructs owner-issued leases");
        let mut recoverable = recover_online_backups(selected);
        assert_eq!(recoverable.len(), 1);
        let resumed = recoverable
            .next()
            .expect("active backup")
            .readmit(&authority, &backup_custody(&authority), 4 * 1024)
            .expect("child readmits the exact cut from durable source observations");
        assert_eq!(
            resumed.cut().identity(),
            resumed.cut().lease().cut_identity()
        );
        let protected = reclaim_reference(BackupArtifactFamily::Page, 4);
        let evidence = ExecutedReachabilityEvidence::for_certification_reference(protected);
        let hazards = HazardLeaseTable::with_capacity(
            HazardLeaseTableCapacity::bounded_slots(1).expect("capacity"),
        )
        .live_index_snapshot();
        let proof = ReclaimEligibilityProof::admit(
            evidence,
            hazards,
            leases.live_index_snapshot().expect("recovered lease index"),
        )
        .expect("reclaim decision");
        assert!(matches!(
            proof.try_reclaim(),
            Err(ReclaimDenial::BlockedByBackupCut { .. })
        ));
        return;
    }

    let scenario = BackupScenario::new("lease-recovery");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-lease-recovery").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut lease");
    let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .arg("--exact")
        .arg("phase_1_6_tests::control_plane::fresh_process_recovers_the_physical_reclaim_interlock_from_control_media")
        .arg("--nocapture")
        .env(CHILD_CONTROL, &scenario.control)
        .status()
        .expect("fresh lease recovery process");
    assert!(status.success());

    admitted
        .abandon("lease recovery closeout", &control, &scenario.leases)
        .expect("durable release");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(2).expect("release generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing) else {
        panic!("released state must be selected");
    };
    assert_eq!(
        selected
            .recover_backup_reachability_leases()
            .expect("replay released state")
            .live_index_snapshot()
            .expect("replayed lease index")
            .active_leases(),
        0
    );
}

#[test]
fn recovered_cut_refuses_source_mutation_instead_of_minting_resume_authority() {
    let scenario = BackupScenario::new("mutated-recovery-source");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-mutated-recovery-source").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut lease");
    let cut_identity = admitted.cut().identity();
    drop(admitted);
    std::fs::write(
        scenario.references()[0].source_path(),
        b"replacement bytes with a different physical cut",
    )
    .expect("mutate cut source");
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(1).expect("lease generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);
    let ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing) else {
        panic!("durable control state must remain readable");
    };
    let recovery = recover_online_backups(selected).next().expect("one backup");

    let denial = recovery
        .readmit(&authority, &backup_custody(&authority), 4 * 1024)
        .expect_err("mutated source cannot be readmitted");
    assert!(matches!(
        denial.source(),
        OnlineBackupReadmissionFailure::Cut(
            BackupCutReadmissionDenial::SourceLengthChanged(_)
                | BackupCutReadmissionDenial::SourceDigestChanged(_)
                | BackupCutReadmissionDenial::SourcePhysicalIdentityChanged(_)
        )
    ));
    let (retry, _) = denial.into_retry();
    assert_eq!(retry.cut_identity(), cut_identity);
}

#[test]
fn selected_control_state_fails_closed_when_its_lease_object_is_missing() {
    let scenario = BackupScenario::new("missing-lease-object");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let _admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-missing-lease-object").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable cut lease");
    let object_root = scenario.control.with_file_name("operations.log.objects");
    for entry in std::fs::read_dir(object_root).expect("lease object directory") {
        std::fs::remove_file(entry.expect("lease object entry").path())
            .expect("remove lease object");
    }
    let selection = TestControlStoreFencingProvider::selected(
        &authority,
        &control,
        ControlStoreGeneration::from_raw(1).expect("lease generation"),
    );
    let fencing = ControlStoreFencingAuthority::for_current_store(&authority, &selection);

    assert!(matches!(
        control.inspect_generations(&fencing),
        ControlStoreTrustPosture::Damaged(
            worth_store_physical_backend::ControlMediaFault::MissingRecoveryObject { .. }
        )
    ));
}
