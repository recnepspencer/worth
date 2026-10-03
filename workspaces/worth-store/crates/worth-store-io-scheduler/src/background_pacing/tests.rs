mod interference_evidence;
mod pacing_edges;
mod policy_receipts;
mod revocation_units;
mod secure_io_preservation;
mod test_support;

use core::num::NonZeroU64;

use test_support::{background_budget_with_queue_slots, read_pressure_budget, World};

use crate::{
    BackgroundDebtKind, BackgroundIoPressureClass, BackgroundIoPressureShape,
    BackgroundPacingDenial, BackgroundPacingOutcome, IoSchedulerBackendCapabilityRequirement,
    QueueSlot,
};

#[test]
fn filesystem_checkpoint_shape_requires_filesystem_admitted_fsync_evidence() {
    assert_eq!(
        BackgroundIoPressureShape::filesystem_admitted_checkpoint_flush().backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedFsync
    );
}

#[test]
fn buffered_arena_compaction_does_not_claim_direct_io() {
    let buffered = BackgroundIoPressureShape::buffered_file_compaction_rewrite();
    assert_eq!(
        buffered.class(),
        BackgroundIoPressureClass::CompactionRewrite
    );
    assert_eq!(
        buffered.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::BufferedFile
    );
    assert_eq!(
        BackgroundIoPressureShape::compaction_rewrite().backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::DirectIo
    );
}

#[test]
fn store_blob_ingest_uses_buffered_media_without_claiming_async_io() {
    let buffered = BackgroundIoPressureShape::buffered_file_blob_ingest_pressure();
    assert_eq!(buffered.class(), BackgroundIoPressureClass::IngestPressure);
    assert_eq!(
        buffered.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::BufferedFile
    );
    assert_eq!(
        BackgroundIoPressureShape::blob_ingest_pressure().backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::AsyncIo
    );
}

#[test]
fn store_blob_reclaim_uses_buffered_media_and_its_own_pressure_class() {
    let buffered = BackgroundIoPressureShape::buffered_file_blob_reclaim_pressure();
    assert_eq!(
        buffered.class(),
        BackgroundIoPressureClass::BlobReclaimPressure
    );
    assert_eq!(
        buffered.backend_requirement(),
        IoSchedulerBackendCapabilityRequirement::BufferedFile
    );
}

#[test]
fn background_pressure_classes_are_distinct_physical_shapes() {
    let classes = [
        BackgroundIoPressureShape::compaction_rewrite().class(),
        BackgroundIoPressureShape::checkpoint_flush().class(),
        BackgroundIoPressureShape::scrub_scan().class(),
        BackgroundIoPressureShape::replication_prep_read().class(),
        BackgroundIoPressureShape::blob_ingest_pressure().class(),
        BackgroundIoPressureShape::blob_migration_pressure().class(),
        BackgroundIoPressureShape::blob_reclaim_pressure().class(),
        BackgroundIoPressureShape::backup_prep_read().class(),
        BackgroundIoPressureShape::repair_scan().class(),
        BackgroundIoPressureShape::verification_pressure().class(),
    ];
    assert_eq!(classes.len(), 10);
    assert!(classes.contains(&BackgroundIoPressureClass::CompactionRewrite));
    assert!(classes.contains(&BackgroundIoPressureClass::CheckpointFlush));
    assert!(classes.contains(&BackgroundIoPressureClass::ScrubScan));
    assert!(classes.contains(&BackgroundIoPressureClass::ReplicationPrepRead));
    assert!(classes.contains(&BackgroundIoPressureClass::IngestPressure));
    assert!(classes.contains(&BackgroundIoPressureClass::MigrationPressure));
    assert!(classes.contains(&BackgroundIoPressureClass::BlobReclaimPressure));
    assert!(classes.contains(&BackgroundIoPressureClass::BackupPrepRead));
    assert!(classes.contains(&BackgroundIoPressureClass::RepairScan));
    assert!(classes.contains(&BackgroundIoPressureClass::VerificationPressure));
    assert_eq!(
        BackgroundIoPressureClass::CheckpointFlush.debt_kind(),
        BackgroundDebtKind::CheckpointFlushDebt
    );
    assert_eq!(
        BackgroundIoPressureClass::IngestPressure.debt_kind(),
        BackgroundDebtKind::BlobContention
    );
    assert_eq!(
        BackgroundIoPressureClass::MigrationPressure.debt_kind(),
        BackgroundDebtKind::BlobContention
    );
    assert_eq!(
        BackgroundIoPressureClass::BlobReclaimPressure.debt_kind(),
        BackgroundDebtKind::BlobContention
    );
    assert_eq!(
        BackgroundIoPressureClass::RepairScan.debt_kind(),
        BackgroundDebtKind::RepairPressure
    );
    assert_eq!(
        BackgroundIoPressureClass::VerificationPressure.debt_kind(),
        BackgroundDebtKind::RepairPressure
    );
}

#[test]
fn background_admits_with_debt_and_revocable_lease() {
    let world = World::new();
    let requested = read_pressure_budget();
    let admitted = requested.min_with(background_budget_with_queue_slots(
        QueueSlot::new(1).unwrap(),
    ));
    let debt_limit = requested.debt_after(admitted);

    let outcome = super::admit_background_pacing(world.request_with(
        BackgroundIoPressureShape::compaction_rewrite().requesting(requested),
        admitted,
        admitted,
        debt_limit,
    ));

    let BackgroundPacingOutcome::AdmittedWithDebt(admitted_with_debt) = outcome else {
        panic!("expected admitted-with-debt, got {outcome:?}");
    };
    assert_eq!(
        admitted_with_debt.debt().kind(),
        BackgroundDebtKind::CompactionDebt
    );
    assert_eq!(admitted_with_debt.counters().compaction_debt(), debt_limit);
    let revocation = admitted_with_debt
        .into_lease()
        .revoke_for_foreground_pressure(NonZeroU64::new(1).unwrap());
    assert_eq!(revocation.revoked_budget(), admitted);
    assert_eq!(
        revocation.basis().class(),
        BackgroundIoPressureClass::CompactionRewrite
    );
    assert_eq!(revocation.counters().revoked_budget(), admitted);
    assert_eq!(revocation.counters().revoke_events(), 1);
    assert_eq!(revocation.counters().foreground_pressure_events(), 1);
}

#[test]
fn policy_admitted_capacity_bounds_execution_even_when_idle_exists() {
    let world = World::new();
    let requested = read_pressure_budget();
    let policy_admitted = background_budget_with_queue_slots(QueueSlot::new(1).unwrap());
    let outcome = super::admit_background_pacing(world.request_with(
        BackgroundIoPressureShape::compaction_rewrite().requesting(requested),
        requested,
        policy_admitted,
        crate::BackgroundResourceBudget::new(),
    ));

    let BackgroundPacingOutcome::Throttled(throttled) = outcome else {
        panic!("expected policy throttle, got {outcome:?}");
    };
    assert_eq!(throttled.admitted_budget(), policy_admitted);
    assert!(throttled.throttled_budget().bandwidth_tokens() > 0);
    let lease = throttled
        .into_lease()
        .expect("nonzero policy-admitted capacity must remain move-owned in the throttle");
    assert_eq!(lease.admitted_budget(), policy_admitted);
}

#[test]
fn debt_limit_cannot_mint_execution_lease_without_admitted_capacity() {
    let world = World::new();
    let requested = read_pressure_budget();
    let no_capacity = crate::BackgroundResourceBudget::new();
    let outcome = super::admit_background_pacing(world.request_with(
        BackgroundIoPressureShape::compaction_rewrite().requesting(requested),
        no_capacity,
        requested,
        requested,
    ));

    let BackgroundPacingOutcome::Throttled(throttled) = outcome else {
        panic!("expected debt-only capacity to throttle, got {outcome:?}");
    };
    assert_eq!(throttled.admitted_budget(), no_capacity);
    assert_eq!(throttled.throttled_budget(), requested);
    assert!(
        throttled.into_lease().is_none(),
        "debt without admitted capacity must not mint execution authority"
    );
}

#[test]
fn policy_zero_budget_defers_without_constructing_background_lease() {
    let world = World::new();
    let requested = read_pressure_budget();
    let outcome = super::admit_background_pacing(world.request_with(
        BackgroundIoPressureShape::scrub_scan().requesting(requested),
        requested,
        crate::BackgroundResourceBudget::new(),
        crate::BackgroundResourceBudget::new(),
    ));

    let BackgroundPacingOutcome::Deferred(deferred) = outcome else {
        panic!("expected zero policy budget to defer, got {outcome:?}");
    };
    assert_eq!(deferred.counters().requested(), requested);
    assert_eq!(deferred.counters().deferred_events(), 1);
}

#[test]
fn debt_counters_distinguish_each_background_pressure_family() {
    let compaction = admitted_debt_counters(BackgroundIoPressureShape::compaction_rewrite());
    assert!(compaction.compaction_debt().queue_slots() > 0);

    let checkpoint = admitted_debt_counters(BackgroundIoPressureShape::checkpoint_flush());
    assert!(checkpoint.checkpoint_flush_debt().queue_slots() > 0);

    let scrub = admitted_debt_counters(BackgroundIoPressureShape::scrub_scan());
    assert!(scrub.scrub_pressure().queue_slots() > 0);

    let replication = admitted_debt_counters(BackgroundIoPressureShape::replication_prep_read());
    assert!(replication.replication_prep_pressure().queue_slots() > 0);

    let blob = admitted_debt_counters(BackgroundIoPressureShape::blob_migration_pressure());
    assert!(blob.blob_contention().queue_slots() > 0);

    let backup = admitted_debt_counters(BackgroundIoPressureShape::backup_prep_read());
    assert!(backup.backup_pressure().queue_slots() > 0);

    let repair = admitted_debt_counters(BackgroundIoPressureShape::repair_scan());
    assert!(repair.repair_pressure().queue_slots() > 0);
}

#[test]
fn foreground_pressure_yields_before_background_borrow() {
    let world = World::new();
    let requested = read_pressure_budget();
    let outcome = super::admit_background_pacing(
        world
            .request(BackgroundIoPressureShape::scrub_scan().requesting(requested))
            .with_foreground_pressure_events(1),
    );

    let BackgroundPacingOutcome::Yield(yielded) = outcome else {
        panic!("expected yield, got {outcome:?}");
    };
    assert_eq!(yielded.counters().yield_events(), 1);
    assert_eq!(yielded.counters().foreground_pressure_events(), 1);
}

#[test]
fn partial_idle_capacity_throttles_without_debt_authority() {
    let world = World::new();
    let requested = read_pressure_budget();
    let admitted = background_budget_with_queue_slots(QueueSlot::new(1).unwrap());
    let outcome = super::admit_background_pacing(world.request_with(
        BackgroundIoPressureShape::backup_prep_read().requesting(requested),
        admitted,
        admitted,
        crate::BackgroundResourceBudget::new(),
    ));

    let BackgroundPacingOutcome::Throttled(throttled) = outcome else {
        panic!("expected throttle, got {outcome:?}");
    };
    assert_eq!(throttled.admitted_budget(), admitted);
    assert!(throttled.throttled_budget().bandwidth_tokens() > 0);
    let lease = throttled
        .into_lease()
        .expect("partially admitted idle capacity must remain move-owned in the throttle");
    assert_eq!(lease.admitted_budget(), admitted);
}

#[test]
fn missing_budget_denies_before_pacing_receipt() {
    let world = World::new();
    assert_eq!(
        world.capacity_denial(BackgroundIoPressureShape::repair_scan()),
        BackgroundPacingDenial::MissingDeclaredResourceBudget
    );
}

#[test]
fn secure_scope_pressure_requires_secure_io_before_lease() {
    let world = World::new();
    let requested = read_pressure_budget();
    for pressure in [
        BackgroundIoPressureShape::backup_prep_read().requesting(requested),
        BackgroundIoPressureShape::repair_scan().requesting(requested),
        BackgroundIoPressureShape::verification_pressure().requesting(requested),
    ] {
        assert_eq!(
            world.capacity_denial(pressure),
            BackgroundPacingDenial::MissingSecureIoPreservation
        );
    }
}

#[test]
fn late_yield_records_typed_violation_debt() {
    let world = World::new();
    let requested = read_pressure_budget();
    let admitted = background_budget_with_queue_slots(QueueSlot::new(1).unwrap());
    let debt_limit = requested.debt_after(admitted);
    let outcome = super::admit_background_pacing(
        world
            .request_with(
                BackgroundIoPressureShape::replication_prep_read().requesting(requested),
                admitted,
                admitted,
                debt_limit,
            )
            .with_foreground_pressure_events(1)
            .with_late_yield(),
    );

    let BackgroundPacingOutcome::Violation(violation) = outcome else {
        panic!("expected violation, got {outcome:?}");
    };
    assert_eq!(
        violation.causal_debt().kind(),
        BackgroundDebtKind::ReplicationPrepPressure
    );
    assert_eq!(violation.counters().violation_events(), 1);
    assert_eq!(violation.counters().replication_prep_pressure(), debt_limit);
}

#[test]
fn backend_mismatch_denies_before_background_pacing() {
    let world = World::new();
    let requested = read_pressure_budget();
    assert!(matches!(
        world.capacity_denial(
            BackgroundIoPressureShape::blob_ingest_pressure().requesting(requested)
        ),
        BackgroundPacingDenial::BackendRequirementMismatch { .. }
    ));
}

#[test]
fn raw_and_diagnostic_shortcuts_are_typed_denials() {
    assert_eq!(
        super::reject_raw_background_label_as_background_pacing_authority(),
        Err(BackgroundPacingDenial::RawBackgroundLabelCannotPace)
    );
    assert_eq!(
        super::reject_semantic_lifecycle_receipt_as_background_pacing_authority(),
        Err(BackgroundPacingDenial::SemanticLifecycleReceiptCannotPace)
    );
    assert_eq!(
        super::reject_log_line_as_background_pacing_authority(),
        Err(BackgroundPacingDenial::LogLineCannotPace)
    );
    assert_eq!(
        super::reject_elapsed_time_as_background_pacing_authority(),
        Err(BackgroundPacingDenial::ElapsedTimeCannotPace)
    );
    assert_eq!(
        super::reject_worker_local_queue_as_background_pacing_authority(),
        Err(BackgroundPacingDenial::WorkerLocalQueueCannotPace)
    );
}

fn admitted_debt_counters(
    shape: BackgroundIoPressureShape,
) -> crate::BackgroundPacingCounterSnapshot {
    let world = if shape.class() == BackgroundIoPressureClass::CheckpointFlush {
        World::commit_wal()
    } else {
        World::new()
    };
    let requested = read_pressure_budget();
    let admitted = background_budget_with_queue_slots(QueueSlot::new(1).unwrap());
    let debt_limit = requested.debt_after(admitted);
    let outcome = super::admit_background_pacing(world.request_with(
        shape.requesting(requested),
        admitted,
        admitted,
        debt_limit,
    ));
    let BackgroundPacingOutcome::AdmittedWithDebt(admitted) = outcome else {
        panic!("expected admitted-with-debt, got {outcome:?}");
    };
    admitted.counters()
}
