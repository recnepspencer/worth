#![cfg(feature = "certification-test-authority")]

#[allow(dead_code)]
mod phase_three_support;

use std::path::{Path, PathBuf};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::{
    MediaFaultDirective, MediaFaultSchedule, MediaOperationRole,
};
use worth_store::physical_runtime::{
    ArtifactTreeFailureKind, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
    PhysicalRecoveryStagingCommandIndeterminate, RecoveryStagingIndeterminatePhysical,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStagingSettlement,
    PhysicalRecoveryStaticConfiguration,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_durable_wal_attempt_without_execution, canonical_physical_mutation_acknowledgment,
    PhysicalResidencyStoreWorld,
};

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
        MediaOperationRole::PositionedWrite,
        MediaFaultDirective::AllowPrefix { bytes: 7 },
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
    let [PhysicalRecoveryStagingSettlement::Indeterminate(
        PhysicalRecoveryStagingCommandIndeterminate::Materialization { physical, .. },
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
        RecoveryStagingIndeterminatePhysical::Append {
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

fn assert_existing_artifact_is_damaged(label: &str, bytes: impl FnOnce(&[u8]) -> Vec<u8>) {
    let (parent, root) = ordinary_persisted_world(label);
    let planned = plan_with_schedule(&root, empty_fault_schedule());
    let (artifact, expected) = only_command(&planned);
    std::fs::write(record_artifact_path(&root, artifact), bytes(&expected)).unwrap();

    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) =
        planned.stage()
    else {
        panic!("conflicting existing artifact must block")
    };
    assert!(matches!(
        blocked.evidence().staging_settlements.as_ref().unwrap().entries(),
        [PhysicalRecoveryStagingSettlement::DeniedBeforeEffect(denial)]
            if matches!(
                denial.denial(),
                worth_store::physical_runtime::PhysicalRecoveryStagingCommandDenialKind::Media(
                    failure
                ) if failure.kind() == ArtifactTreeFailureKind::Damaged
            )
    ));
    assert_eq!(blocked.recovery_effects(), 0);
    drop(parent);
}

fn ordinary_persisted_world(label: &str) -> (tempfile::TempDir, PathBuf) {
    let parent = tempfile::tempdir().expect("prefix world parent");
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery(label).unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x81; 32], b"prefix-base");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x83; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("prefix proof checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    canonical_durable_wal_attempt_without_execution(&world, [0x82; 32], b"prefix-redo");
    drop(world);
    let copied = parent.path().join("retained-root");
    copy_directory(&retained_root.persist(), &copied);
    (parent, copied)
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

fn only_command(
    planned: &worth_store_recovery_runtime::PlannedPhysicalRecovery,
) -> (worth_store_physical_format::RecordArtifactFile, Vec<u8>) {
    assert_eq!(planned.staging_layout().commands().len(), 1);
    let command = &planned.staging_layout().commands()[0];
    (command.artifact(), command.bytes().to_vec())
}

fn empty_fault_schedule() -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    admission
        .fault_schedule_authority()
        .schedule(Vec::new())
        .unwrap()
}

fn recovery_fault_schedule(
    role: MediaOperationRole,
    directive: MediaFaultDirective,
) -> MediaFaultSchedule {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    authority
        .schedule(vec![authority.rule(role, 1, directive)])
        .unwrap()
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

fn record_artifact_path(
    root: &Path,
    artifact: worth_store_physical_format::RecordArtifactFile,
) -> PathBuf {
    let family = match artifact {
        worth_store_physical_format::RecordArtifactFile::Segment { .. } => "segments",
        worth_store_physical_format::RecordArtifactFile::Extent { .. } => "extents",
        _ => panic!("prefix proof stages a data artifact"),
    };
    root.join("families/records")
        .join(family)
        .join(artifact.file_name())
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
