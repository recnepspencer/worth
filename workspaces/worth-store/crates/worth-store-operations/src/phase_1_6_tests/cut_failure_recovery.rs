use super::control_append_fault::{
    FailAfterSuccessfulControlAppends, LoseSuccessfulControlAppendReceipt,
};
use super::support::*;

#[test]
fn source_verification_runs_while_a_reachability_reservation_blocks_reclaim() {
    let scenario = BackupScenario::new("source-verification-lease-order");
    let authority = scenario.authority();
    let protected = scenario.references()[0].reclaim_reference();
    let verifier = |manifest: &BackupCutManifest| {
        let evidence = ExecutedReachabilityEvidence::for_certification_reference(protected);
        let hazards = HazardLeaseTable::with_capacity(
            HazardLeaseTableCapacity::bounded_slots(1).expect("capacity"),
        )
        .live_index_snapshot();
        let proof = ReclaimEligibilityProof::admit(
            evidence,
            hazards,
            scenario
                .leases
                .live_index_snapshot()
                .expect("pending lease reservation is visible to reclaim"),
        )
        .expect("reclaim decision");
        assert!(matches!(
            proof.try_reclaim(),
            Err(ReclaimDenial::BlockedByBackupCut { .. })
        ));
        worth_store_offline_verifier::verify_backup_cut_sources(
            manifest,
            OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget"),
        )
    };
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-source-verification-lease-order").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut_with_source_verification(
        &authority,
        &scenario.control_store(),
        &scenario.leases,
        verifier,
    )
    .expect("source verification under protected cut");
    assert_eq!(admitted.source_verification().artifacts_attempted(), 7);
    assert_eq!(admitted.source_verification().artifacts_verified(), 7);
}

#[test]
fn wal_cut_coverage_uses_the_wal_owners_half_open_interval_semantics() {
    let scenario = BackupScenario::new("half-open-wal-cut");
    let authority = scenario.authority();
    let mut artifacts = scenario
        .references()
        .iter()
        .filter(|artifact| artifact.family() != BackupArtifactFamily::WalSegment)
        .cloned()
        .collect::<Vec<_>>();
    for (index, (start, end_exclusive)) in [(10, 11), (11, 12)].into_iter().enumerate() {
        let path = scenario.source.join(format!("half-open-wal-{index}.media"));
        let segment_id = 70 + index as u64;
        let frame = worth_store_wal::prepare_wal_frame_append(
            &scenario.source,
            segment_id,
            1,
            start,
            end_exclusive,
            &format!("half-open-wal-{index}"),
            format!("wal-frame-{start}-{end_exclusive}").as_bytes(),
        )
        .expect("owner-issued WAL frame");
        std::fs::write(&path, frame.encoded_frame()).expect("WAL source");
        artifacts.push(
            BackupArtifactReference::declare_untrusted_physical_observation(
                UntrustedBackupArtifactClaim {
                    family: BackupArtifactFamily::WalSegment,
                    format: artifact_format(BackupArtifactFamily::WalSegment),
                    identity: format!("half-open-wal-{index}"),
                    generation: 1,
                    coverage: BackupArtifactCoverage::wal_segment(start, end_exclusive)
                        .expect("nonempty half-open WAL interval"),
                },
                observe_physical_backup_artifact(path, 4 * 1024).expect("WAL observation"),
                reclaim_reference(BackupArtifactFamily::WalSegment, 70 + index as u16),
            )
            .expect("WAL artifact"),
        );
    }
    assert!(BackupArtifactCoverage::wal_segment(12, 12).is_none());
    let manifest = BackupCutManifest::canonical(artifacts).expect("complete cut");

    OnlineBackupIntent::new(
        OperationalOperationId::new("backup-half-open-wal-cut").expect("operation"),
        scenario.coordinates(),
        manifest,
        backup_custody(&authority),
    )
    .admit_cut(&authority, &scenario.control_store(), &scenario.leases)
    .expect("10..11 and 11..12 form exact 10..12 WAL coverage");
}

#[test]
fn published_bundle_survives_control_receipt_failure_as_a_retryable_linear_state() {
    let scenario = BackupScenario::new("receipt-retry");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-receipt-retry").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut");
    let fail_materialization_receipt = FailAfterSuccessfulControlAppends::new(&control, 1);
    let denial = admitted
        .materialize(&scenario.target, 7, &fail_materialization_receipt)
        .expect("session")
        .finish()
        .expect_err("receipt append must fail");
    let BackupMaterializationDenial::Control(denial) = denial else {
        panic!("expected a retryable control denial");
    };
    let (unrecorded, _) = (*denial).into_retry();
    assert!(unrecorded.bundle().root().exists());

    let completed = unrecorded.record(&control).expect("retry durable receipt");
    assert!(completed.bundle().root().exists());
}

#[test]
fn admitted_cut_survives_lease_receipt_failure_as_a_retryable_linear_state() {
    let scenario = BackupScenario::new("lease-retry");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let fault = ObserveReservedLeaseThenFail {
        delegate: &control,
        leases: &scenario.leases,
        calls: std::cell::Cell::new(0),
    };
    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-lease-retry").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &fault, &scenario.leases)
    .expect_err("lease receipt must fail");
    let OnlineBackupAdmissionDenial::LeasePersistence(denial) = denial else {
        panic!("expected retryable lease persistence denial");
    };
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("rolled-back reservation")
            .active_leases(),
        0
    );
    let (unpersisted, _) = denial.into_retry();
    let _admitted = unpersisted
        .persist(&control, &scenario.leases)
        .expect("retry lease receipt");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("lease registry")
            .active_leases(),
        1
    );
}

#[test]
fn lost_successful_lease_receipt_retries_the_exact_holder_without_under_or_over_counting() {
    let scenario = BackupScenario::new("lost-successful-lease-receipt");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let fault = LoseSuccessfulControlAppendReceipt::new(&control, 0);
    let denial = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-lost-successful-lease-receipt").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &fault, &scenario.leases)
    .expect_err("durable receipt is deliberately hidden from the caller");
    let OnlineBackupAdmissionDenial::LeasePersistence(denial) = denial else {
        panic!("lost receipt must preserve the unpersisted linear state");
    };
    let after_loss = scenario
        .leases
        .live_index_snapshot()
        .expect("rolled-back in-memory reservation");
    assert_eq!(after_loss.active_leases(), 0);
    assert_eq!(after_loss.active_holders(), 0);

    let (unpersisted, _) = denial.into_retry();
    let admitted = unpersisted
        .persist(&control, &scenario.leases)
        .expect("idempotent durable append reconstructs the missing holder");
    let after_retry = scenario
        .leases
        .live_index_snapshot()
        .expect("exact holder after retry");
    assert_eq!(after_retry.active_leases(), 1);
    assert_eq!(after_retry.active_holders(), 1);

    admitted
        .abandon("lost receipt closeout", &control, &scenario.leases)
        .expect("exact holder releases once");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("released holder")
            .active_holders(),
        0
    );
}

#[test]
fn lost_successful_release_receipt_keeps_the_exact_holder_retryable_until_acknowledged() {
    let scenario = BackupScenario::new("lost-successful-release-receipt");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let admitted = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-lost-successful-release-receipt").expect("operation"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("durable holder");
    let fault = LoseSuccessfulControlAppendReceipt::new(&control, 0);
    let denial = admitted
        .abandon("lost release receipt", &fault, &scenario.leases)
        .expect_err("durable release receipt is deliberately hidden");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("release remains unacknowledged in memory")
            .active_holders(),
        1
    );

    let (retry, _) = denial.into_retry();
    retry
        .abandon("lost release receipt", &control, &scenario.leases)
        .expect("idempotent release acknowledges the exact holder");
    assert_eq!(
        scenario
            .leases
            .live_index_snapshot()
            .expect("release acknowledged")
            .active_holders(),
        0
    );
}
