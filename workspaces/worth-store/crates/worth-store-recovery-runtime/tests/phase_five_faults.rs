#![cfg(feature = "certification-test-authority")]

use std::path::{Path, PathBuf};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::{
    MediaFaultDirective, MediaFaultSchedule, MediaOperationRole,
};
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    RecoveryStagingIndeterminatePhysical,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

#[test]
fn partial_materialization_retains_the_exact_indeterminate_receipt() {
    let (parent, root) = ordinary_persisted_world("partial-materialization");
    let schedule = recovery_fault_schedule(
        MediaOperationRole::PositionedWrite,
        MediaFaultDirective::AllowPrefix { bytes: 31 },
    );
    let blocked = expect_staging_block(plan_with_schedule(&root, schedule).stage());
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 1);
    assert_eq!(counters.commands_settled, 1);
    assert_eq!(counters.scheduler_settlements, 1);
    assert_eq!(counters.performed_effects, 0);
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::Indeterminate {
            ordinal: 0,
            stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization,
        })
    ));
    let [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::Indeterminate(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate::Materialization {
            physical,
            scheduler,
        },
    )] = blocked
        .evidence()
        .staging_settlements
        .as_ref()
        .unwrap()
        .entries()
    else {
        panic!("partial write retains one exact indeterminate settlement")
    };
    assert_eq!(
        *scheduler,
        Some(worth_store::physical_runtime::PhysicalWorkSchedulerPosture::Executed)
    );
    assert!(matches!(
        physical.evidence(),
        RecoveryStagingIndeterminatePhysical::NewArtifact(new_artifact)
            if new_artifact.completed_bytes() == 31
                && new_artifact.create_operation().value() != 0
                && new_artifact.write_operation().is_some()
    ));
    assert_eq!(blocked.recovery_effects(), 1);
    drop(parent);
}

#[test]
fn synchronization_indeterminacy_retains_materialization_and_barrier_occurrence() {
    let (parent, root) = ordinary_persisted_world("sync-indeterminate");
    let schedule = recovery_fault_schedule(
        MediaOperationRole::SynchronizeFileState,
        MediaFaultDirective::IndeterminateAfterEffect,
    );
    let blocked = expect_staging_block(plan_with_schedule(&root, schedule).stage());
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(counters.commands_submitted, 2);
    assert_eq!(counters.commands_settled, 2);
    assert_eq!(counters.scheduler_settlements, 2);
    assert_eq!(counters.artifacts_created, 1);
    assert_eq!(counters.artifacts_synchronized, 0);
    assert_eq!(counters.performed_effects, 1);
    let [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::Indeterminate(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate::Synchronization {
            physical,
            materialization,
            scheduler,
        },
    )] = blocked
        .evidence()
        .staging_settlements
        .as_ref()
        .unwrap()
        .entries()
    else {
        panic!("file barrier retains materialization and exact indeterminate effect")
    };
    assert_eq!(
        *scheduler,
        Some(worth_store::physical_runtime::PhysicalWorkSchedulerPosture::Executed)
    );
    assert_eq!(
        materialization.physical().byte_count(),
        counters.bytes_written
    );
    assert_ne!(physical.operation().value(), 0);
    assert!(physical.effect().is_file_synchronization());
    assert_eq!(blocked.recovery_effects(), 2);
    drop(parent);
}

#[test]
fn materialization_signal_derivation_failure_cannot_close_staging() {
    assert_signal_derivation_failure(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization,
    );
}

#[test]
fn synchronization_signal_derivation_failure_cannot_close_staging() {
    assert_signal_derivation_failure(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization,
    );
}

#[test]
fn ambiguous_effect_wins_over_the_next_cancellation_safe_point() {
    let (parent, root) = ordinary_persisted_world("cancel-after-ambiguous-effect");
    let schedule = recovery_fault_schedule(
        MediaOperationRole::PositionedWrite,
        MediaFaultDirective::IndeterminateAfterEffect,
    );
    let planned = plan_with_schedule(&root, schedule);
    let cancellation = planned.cancellation_after_command(0).unwrap();
    let blocked = expect_staging_block(planned.stage_with_cancellation(cancellation));
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::Indeterminate {
            ordinal: 0,
            stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization,
        })
    ));
    assert_eq!(
        blocked
            .evidence()
            .staging_settlements
            .as_ref()
            .unwrap()
            .entries()
            .len(),
        1
    );
    assert_eq!(
        blocked
            .evidence()
            .staging_counters
            .unwrap()
            .performed_effects,
        0
    );
    drop(parent);
}

fn assert_signal_derivation_failure(
    expected_stage: worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage,
) {
    let label = match expected_stage {
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization => {
            "signal-derivation-materialization"
        }
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization => {
            "signal-derivation-synchronization"
        }
    };
    let (parent, root) = ordinary_persisted_world(label);
    let planned = plan_with_schedule(&root, empty_recovery_fault_schedule());
    planned.certification_fail_staging_signal_settlement_at(expected_stage);
    let blocked = expect_staging_block(planned.stage());
    let counters = blocked.evidence().staging_counters.unwrap();
    assert_eq!(
        counters.commands_submitted,
        match expected_stage {
            worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Materialization =>
                1,
            worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization =>
                2,
        },
        "the exact physical stages completed before Signal derivation failed",
    );
    assert_eq!(counters.commands_settled, counters.commands_submitted);
    assert_eq!(counters.scheduler_settlements, counters.commands_submitted);
    assert_eq!(
        counters.performed_effects,
        u64::from(
            expected_stage
                == worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization
        ),
        "only the earlier terminal materialization may carry performed authority",
    );
    assert_eq!(
        (
            counters.pending_signal_reconciliations_after_close,
            counters.signal_reconciliation_overflow_after_close,
        ),
        (1, 0),
    );
    let [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::Indeterminate(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate::Signal {
            stage,
            materialization,
            synchronization,
            outcome,
        },
    )] = blocked
        .evidence()
        .staging_settlements
        .as_ref()
        .unwrap()
        .entries()
    else {
        panic!("derived-state loss retains one exact Signal settlement")
    };
    assert_eq!(*stage, expected_stage);
    assert_eq!(
        *outcome,
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
    );
    let materialization = materialization
        .as_ref()
        .expect("physical materialization truth is retained");
    assert_eq!(
        materialization.is_performed(),
        expected_stage
            == worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization,
        "the failing stage cannot mint performed authority",
    );
    assert_eq!(
        synchronization.is_some(),
        expected_stage
            == worth_store::physical_runtime::PhysicalRecoveryStagingCommandStage::Synchronization,
    );
    assert!(matches!(
        blocked.evidence().staging_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryStagingDenial::Indeterminate {
            stage,
            ..
        }) if stage == expected_stage
    ));
    drop(parent);
}

fn ordinary_persisted_world(label: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().expect("fault world parent");
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x61; 32], b"fault-base");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x62; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("fault-world checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x63; 32], b"fault-redo");
    drop(world);
    let copied = parent.path().join("retained-root");
    copy_directory(&retained_root.persist(), &copied);
    (parent, copied)
}

fn recovery_fault_schedule(
    role: MediaOperationRole,
    directive: MediaFaultDirective,
) -> MediaFaultSchedule {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    authority
        .schedule(vec![authority.rule(role, 1, directive)])
        .expect("valid recovery fault schedule")
}

fn empty_recovery_fault_schedule() -> MediaFaultSchedule {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    admission
        .fault_schedule_authority()
        .schedule(Vec::new())
        .expect("empty recovery fault schedule")
}

fn plan_with_schedule(
    root: &Path,
    schedule: MediaFaultSchedule,
) -> worth_store_recovery_runtime::PlannedPhysicalRecovery {
    let limits = ordinary_limits();
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let authority = PhysicalRecoveryPlatformAuthority::acquire_for_certification(
        root.to_path_buf(),
        configuration.clone(),
        limits,
        schedule,
    )
    .expect("certification recovery authority");
    let profile = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
    .admit()
    .unwrap()
    .discover()
    .unwrap()
    .select()
    .unwrap()
    .plan()
    .unwrap()
}

fn expect_staging_block(
    outcome: Result<
        worth_store_recovery_runtime::StagedPhysicalRecovery,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    >,
) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) = outcome
    else {
        panic!("faulted staging terminates Blocked")
    };
    blocked
}

fn ordinary_limits() -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 2,
        checkpoint_candidates: 8,
        manifest_bytes: 2 * 1024 * 1024,
        manifest_entries: 4_096,
        wal_segments: 8,
        wal_frames: 4_096,
        wal_bytes: 2 * 1024 * 1024,
        redo_targets: 4_096,
        redo_bytes: 4 * 1024 * 1024,
        distinct_pages_and_extents: 4_096,
        operation_bindings: 4_096,
        staging_bytes: 32 * 1024 * 1024,
        recovery_memory_bytes: 32 * 1024 * 1024,
        dirty_frames: 4_096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4_096,
        cleanup_bytes: 2 * 1024 * 1024,
        observation_bytes: 32 * 1024 * 1024,
    })
    .unwrap()
}

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
