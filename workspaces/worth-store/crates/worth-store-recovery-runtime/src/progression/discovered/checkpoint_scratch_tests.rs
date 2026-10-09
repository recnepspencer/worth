//! Real NoRelease checkpoint selection requires native canonical-root scratch.

use std::{
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalOperationAllocationScope as Scope,
    PhysicalRecoveryRejoinResidentDenial, PhysicalResidencyDimension as Dimension,
};
use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryRootProtocolArtifact as Artifact, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause, PhysicalRecoveryStaticConfiguration,
};
use crate::orchestration::CheckpointDiscovery;

const ORIGINAL_BYTES: u64 = 16 << 20;

#[test]
fn genuine_checkpoint_source_root_scratch_denies_live_pressure_then_selects_after_release() {
    std::thread::Builder::new()
        .name("checkpoint-source-scratch-pressure".to_owned())
        .stack_size(16 << 20)
        .spawn(run_source_root_scratch_pressure)
        .unwrap()
        .join()
        .unwrap();
}

fn run_source_root_scratch_pressure() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("source-root-scratch")
        .expect("real initialized Store Serving");
    world
        .serving()
        .certification_activate_tier_epoch(world.placement())
        .expect("real root-only durable publication before the empty NoRelease checkpoint");
    let store = world.serving().store_identity();
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xd8; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("ordinary Store checkpoint request must admit");
    };
    let PhysicalCheckpointOutcome::Completed(_) = handle.wait() else {
        panic!("ordinary Store checkpoint must publish its actual NoRelease custody");
    };
    let retained = world.retained_root();
    let root = retained.path();
    drop(world);

    let mut discovered = recovery_request(root).admit().unwrap().discover().unwrap();
    let (generation, shared_bytes, source_bytes) = match &discovered.material.checkpoint {
        CheckpointDiscovery::Admitted {
            projection,
            source_root,
        } => (
            projection.checkpoint.facts().source().root().generation(),
            projection.checkpoint.owned_heap_bytes().unwrap(),
            source_root.charged_bytes(),
        ),
        _ => panic!("real Store checkpoint must be admitted by C4 discovery"),
    };
    assert!(shared_bytes > 0 && source_bytes > 0);
    let media_before = selected_media_digests(root, generation);
    let owner = discovered.material.coordination.owner_mut();
    let allocations = owner.certification_residency_allocations();
    let recovery = Dimension::OperationScope(Scope::Recovery);
    let active = allocations
        .snapshot()
        .for_dimension(recovery)
        .active_units();
    assert!(
        active >= shared_bytes + source_bytes,
        "both the retained checkpoint and actual source-root read remain funded before selection"
    );
    let scratch = DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64;
    let held_bytes = ORIGINAL_BYTES
        .checked_sub(active)
        .and_then(|remaining| remaining.checked_sub(scratch - 1))
        .expect("actual native occupancy leaves room for an independently held competing grant");
    let held = owner
        .certification_begin_recovery_allocation(NonZeroU64::new(held_bytes).unwrap())
        .expect("actual original Recovery ceiling admits all but one byte of the required scratch");
    assert_eq!(
        allocations
            .snapshot()
            .for_dimension(recovery)
            .active_units(),
        ORIGINAL_BYTES - scratch + 1
    );

    let Err(PhysicalRecoveryOutcome::Blocked(blocked)) = discovered.select() else {
        panic!("checkpoint-source canonical allocation must deny before source-root ingress");
    };
    assert_eq!(blocked.store_identity(), store);
    assert_eq!(blocked.recovery_effects(), 0);
    let [PhysicalRecoverySourceDenial::SourceReadAllocation {
        artifact:
            Artifact::CheckpointSourceRoot {
                generation: observed_generation,
            },
        boundary: Boundary::CanonicalValidation,
        requested,
        cause:
            Cause::Residency(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                required,
                admitted,
            }),
    }] = blocked.evidence().source_denials.as_slice()
    else {
        panic!(
            "canonical scratch refusal must retain its exact original/native cause: {:?}",
            blocked.evidence().source_denials
        );
    };
    assert_eq!(*observed_generation, generation);
    assert_eq!(*requested, scratch);
    assert_eq!(*required, ORIGINAL_BYTES + 1);
    assert_eq!(*admitted, ORIGINAL_BYTES);
    assert_eq!(selected_media_digests(root, generation), media_before);
    let observation_bytes = blocked.evidence().integrity_observations.charged_bytes();
    assert!(
        observation_bytes > 0,
        "blocked diagnostics retain the actual WAL observation owner"
    );
    assert_eq!(
        allocations
            .snapshot()
            .for_dimension(recovery)
            .active_units(),
        held_bytes + observation_bytes,
        "blocked material disposes Shared and source reads while retaining diagnostic storage"
    );
    drop(held);
    assert_eq!(
        allocations
            .snapshot()
            .for_dimension(recovery)
            .active_units(),
        observation_bytes,
        "only the blocked outcome's shared observations retain Recovery storage"
    );
    drop(blocked);
    let disposed = allocations.snapshot().for_dimension(recovery);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
    let disposed = allocations.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());

    let selected = recovery_request(root)
        .admit()
        .unwrap()
        .discover()
        .unwrap()
        .select()
        .expect("identical healthy bounded request selects the same actual checkpoint");
    drop(selected);
    assert_eq!(selected_media_digests(root, generation), media_before);
}

fn recovery_request(root: &Path) -> PhysicalRecoveryOpenRequest {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 64 << 20,
        manifest_entries: 4096,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 64 << 20,
        redo_targets: 4096,
        redo_bytes: 64 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 64 << 20,
        recovery_memory_bytes: ORIGINAL_BYTES,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 64 << 20,
        observation_bytes: 64 << 20,
    })
    .unwrap();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .expect("fresh production entry authority for the unchanged persisted namespace");
    let backend = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(root.to_owned(), configuration, backend, limits, authority)
}

fn selected_media_digests(root: &Path, generation: u64) -> [(PathBuf, [u8; 32]); 3] {
    let records = root.join("families/records");
    [
        root.join("families/checkpoint.current"),
        records.join(RecordArtifactFile::CurrentRootSelector.file_name()),
        records
            .join("roots")
            .join(RecordArtifactFile::RootManifest { generation }.file_name()),
    ]
    .map(|path| {
        let bytes = std::fs::read(&path).expect("actual selected media remains readable");
        let digest = Sha256::digest(bytes).into();
        (path, digest)
    })
}
