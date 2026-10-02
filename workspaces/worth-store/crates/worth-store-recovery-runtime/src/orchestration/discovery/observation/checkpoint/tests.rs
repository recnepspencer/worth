use std::num::NonZeroU64;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, FilesystemAccessPosture,
    FilesystemMediaAdmission, PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort,
    PhysicalRuntimeAdmission, PhysicalStore, QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointRootBasis, CheckpointStreamEncoder,
    CheckpointWalSourceRange, PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};
use worth_store_recovery_physics::PhysicalCheckpointBaseDenial;

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits,
    PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::progression::PhysicalRecoveryDiscoveryCounters;

#[test]
fn resealed_checkpoint_zero_root_is_denied_before_addressed_source_read() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("zero-source-root");
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("production media admission")
    };
    let store = media.store_identity();
    media.close();
    let identity = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(7).unwrap());
    let source = PhysicalCheckpointSource::concurrent(
        identity,
        CheckpointWalSourceRange::new(10, 20).unwrap(),
        CheckpointRootBasis::new(0, 4),
        5,
    );
    // Canonical encoding re-establishes every CRC/security binding around the hostile address.
    let (encoder, mut bytes) = CheckpointStreamEncoder::begin(source);
    let (compaction, header) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 20).unwrap());
    let (_, footer) = compaction.finish();
    bytes.extend(header);
    bytes.extend(footer);
    let families = root.join("families");
    std::fs::create_dir_all(&families).unwrap();
    std::fs::write(families.join("checkpoint.current"), &bytes).unwrap();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(&root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let mut coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(1, 4096, 1, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(4096)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    #[cfg(feature = "certification-test-authority")]
    let observer = coordination.certification_residency_allocations();
    let mut allocation = coordination.begin_source_read_allocation().unwrap();
    // Only the checkpoint read fits: a source read would consume a second discovery entry.
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let mut remaining_manifest_bytes = 4096;
    let mut counters = PhysicalRecoveryDiscoveryCounters::default();
    let mut trace = RecoveryIntegrityIngressTrace::new();
    let failure = match super::observe_checkpoint(
        &mut discovery,
        limits(),
        &mut remaining_manifest_bytes,
        &mut counters,
        &mut trace,
        &mut allocation,
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("zero source address must not become checkpoint material"),
    };
    assert_eq!(failure.kind, PhysicalRecoveryBlockKind::Checkpoint);
    assert!(matches!(
        failure.source_denials.as_slice(),
        [PhysicalRecoverySourceDenial::CheckpointBinding(
            PhysicalCheckpointBaseDenial::RootGenerationMismatch
        )]
    ));
    assert_eq!(remaining_manifest_bytes, 4096);
    assert_eq!(discovery.counters().bytes_read, bytes.len() as u64);
    assert_eq!(
        allocation.charged_bytes(),
        0,
        "this zero-record fixture requires no parser allocation"
    );
    #[cfg(feature = "certification-test-authority")]
    {
        use worth_store::physical_runtime::{
            PhysicalOperationAllocationScope, PhysicalResidencyDimension,
        };
        let native = observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::OperationScope(
                PhysicalOperationAllocationScope::Recovery,
            ));
        assert_eq!(
            native.admissions(),
            3,
            "checkpoint file address, capability open and payload; never source or Shared retention"
        );
        assert!(native.admitted_units() > bytes.len() as u64);
        assert_eq!(native.released_units(), native.admitted_units());
        assert_eq!(native.active_units(), 0);
    }
    let ingress = failure.integrity_trace.counters();
    assert_eq!(
        ingress.attempted, 5,
        "envelope plus ordered body admissions"
    );
    assert_eq!(
        ingress.admitted, 5,
        "all resealed records pass byte integrity"
    );
    discovery.finish();
}

fn limits() -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 2,
        checkpoint_candidates: 1,
        manifest_bytes: 4096,
        manifest_entries: 1,
        wal_segments: 1,
        wal_frames: 1,
        wal_bytes: 4096,
        redo_targets: 1,
        redo_bytes: 4096,
        distinct_pages_and_extents: 1,
        operation_bindings: 1,
        staging_bytes: 4096,
        recovery_memory_bytes: 4096,
        dirty_frames: 1,
        concurrent_commands: 1,
        publication_effects: 1,
        cleanup_candidates: 1,
        cleanup_bytes: 4096,
        observation_bytes: 4096,
    })
    .unwrap()
}
