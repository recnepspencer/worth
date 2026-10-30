#![cfg(all())]

#[allow(unused_imports)]
use crate::phase_five_faults::*;
use {std::path::Path, std::path::PathBuf};

use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaFaultSchedule,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
use {
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::RecoveryStagingIndeterminatePhysical,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
};
use {
    worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution,
    worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

#[test]
fn partial_materialization_retains_the_exact_indeterminate_receipt() {
    let (parent, root) = ordinary_persisted_world("partial-materialization");
    let schedule = recovery_fault_schedule(
        worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite,
        worth_store::physical_runtime::certification::MediaFaultDirective::AllowPrefix {
            bytes: 31,
        },
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
        worth_store::physical_runtime::RecoveryStagingIndeterminatePhysical::NewArtifact(new_artifact)
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
        worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileState,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
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
        worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
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
