#![cfg(all())]



#[allow(unused_imports)] use crate::phase_five_prefix_completion::*;
use {std::path::Path, std::path::PathBuf};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaFaultSchedule, worth_store::physical_runtime::certification::MediaOperationRole};
use {worth_store::physical_runtime::ArtifactTreeFailureKind, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate, worth_store::physical_runtime::RecoveryStagingIndeterminatePhysical};
use {worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration, worth_store_recovery_runtime::PhysicalRecoveryLimits, worth_store_recovery_runtime::PhysicalRecoveryOpenRequest, worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority, worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration};
use {worth_store_test_support::harness::physical_residency::canonical_durable_wal_attempt_without_execution, worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[test]
fn exact_prefix_completion_is_performed_and_counted_separately_from_convergence() {
    let (parent, root) = ordinary_persisted_world("exact-prefix-completion");
    let planned = plan_with_schedule(&root, empty_fault_schedule());
    let (artifact, expected) = only_command(&planned);
    let prefix_length = expected.len() - 1;
    std::fs::write(
        record_artifact_path(&root, artifact),
        &expected[..prefix_length],
    )
    .unwrap();

    let staged = planned.stage().unwrap_or_else(|error| {
        panic!("MUTANT_PREDICATE:c8-prefix-completion-rejects-valid-shorter-artifact\n{error:?}")
    });
    let counters = staged.staging_counters();
    assert_eq!(counters.artifacts_created, 0);
    assert_eq!(counters.artifacts_converged, 0);
    assert_eq!(counters.artifacts_completed_from_prefix, 1);
    assert_eq!(counters.bytes_verified, prefix_length as u64);
    assert_eq!(counters.bytes_written, 1);
    assert_eq!(
        std::fs::read(record_artifact_path(&root, artifact)).unwrap(),
        expected
    );
    assert!(staged.is_quiescent());
    drop(parent);
}

#[test]
fn mismatched_and_overlong_existing_artifacts_remain_damaged() {
    assert_existing_artifact_is_damaged("prefix-mismatch", |expected| {
        let mut bytes = expected[..expected.len() - 1].to_vec();
        bytes[0] ^= 1;
        bytes
    });
    assert_existing_artifact_is_damaged("prefix-overlong", |expected| {
        let mut bytes = expected.to_vec();
        bytes.push(0);
        bytes
    });
}

#[test]
fn append_indeterminacy_retains_typed_prefix_and_suffix_receipts() {
    let (parent, root) = ordinary_persisted_world("exact-prefix-indeterminate");
    let schedule = recovery_fault_schedule(
        worth_store::physical_runtime::certification::MediaOperationRole::PositionedWrite,
        worth_store::physical_runtime::certification::MediaFaultDirective::AllowPrefix { bytes: 7 },
    );
    let planned = plan_with_schedule(&root, schedule);
    let (artifact, expected) = only_command(&planned);
    let prefix_length = expected.len() - 64;
    std::fs::write(
        record_artifact_path(&root, artifact),
        &expected[..prefix_length],
    )
    .unwrap();

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) =
        planned.stage()
    else {
        panic!("append uncertainty must retain an inspection block")
    };
    let [worth_store_recovery_runtime::PhysicalRecoveryStagingSettlement::Indeterminate(
        worth_store::physical_runtime::PhysicalRecoveryStagingCommandIndeterminate::Materialization { physical, .. },
    )] = blocked
        .evidence()
        .staging_settlements
        .as_ref()
        .unwrap()
        .entries()
    else {
        panic!("append uncertainty retains one materialization settlement")
    };
    assert!(matches!(
        physical.evidence(),
        worth_store::physical_runtime::RecoveryStagingIndeterminatePhysical::Append {
            prefix_verified: Some(prefix),
            append,
        } if prefix.completed_bytes() == prefix_length as u64
            && append.range().offset() == prefix_length as u64
            && append.range().byte_count() == 64
            && append.completed_bytes() == 7
    ));
    assert_eq!(blocked.recovery_effects(), 1);
    drop(parent);
}
