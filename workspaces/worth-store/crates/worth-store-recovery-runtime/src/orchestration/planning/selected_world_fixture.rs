//! A real store, one mutation and one checkpoint deep, admitted for recovery
//! and stopped once its source root is selected.

use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointRequest,
};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits,
    PhysicalRecoveryOpenRequest, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration,
};

pub(super) struct SelectedWorld {
    authority: AdmittedPlatformAuthority,
    coordination: crate::orchestration::RecoveryCoordination,
    root: DurablePhysicalRootManifest,
    pub(super) placements: Vec<CurrentPhysicalRecordPlacement>,
    retained: worth_store_test_support::TemporaryDirectory,
}

/// What a test reads of a selected world through one bounded reader.
pub(super) struct SelectedSource<'a> {
    pub(super) discovery: &'a mut BoundedRecoveryFilesystemDiscovery,
    pub(super) root: &'a DurablePhysicalRootManifest,
    pub(super) placements: &'a [CurrentPhysicalRecordPlacement],
    pub(super) format: PhysicalRecordFormatDeclaration,
    /// The store directory.
    pub(super) store: &'a Path,
}

impl SelectedWorld {
    /// Reads the world, then lets go of it the way a refused recovery does.
    pub(super) fn read<T>(self, read: impl FnOnce(SelectedSource<'_>) -> T) -> T {
        let Self {
            authority,
            coordination,
            root,
            placements,
            retained,
        } = self;
        let AdmittedPlatformAuthority {
            media,
            session,
            _world_binding,
            ..
        } = authority;
        let mut discovery = media.bounded_discovery(64, 1024 * 1024).unwrap();
        let read = read(SelectedSource {
            discovery: &mut discovery,
            root: &root,
            placements: &placements,
            format: PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            store: retained.path(),
        });
        drop(discovery.finish());
        assert!(coordination.shutdown_is_quiescent());
        session.refuse();
        drop(retained);
        read
    }
}

pub(super) fn selected_world(name: &str, segment_pages: u32) -> SelectedWorld {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery_with_segment_pages(
        name,
        segment_pages,
    )
    .unwrap();
    let retained = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x81; 32], &vec![7; 3_000]);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x82; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("selected-world checkpoint admission")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    drop(world);
    let admitted = admitted_recovery(retained.path());
    let selected = admitted.discover().unwrap().select().unwrap();
    let (authority, coordination, selection, _, _, _, _) = selected.into_parts();
    SelectedWorld {
        authority,
        coordination,
        root: selection.root().selected().manifest().clone(),
        placements: selection.page_facts().placements().to_vec(),
        retained,
    }
}

fn admitted_recovery(root: &Path) -> crate::AdmittedPhysicalRecovery {
    let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 4,
        manifest_bytes: 1024 * 1024,
        manifest_entries: 4_096,
        wal_segments: 8,
        wal_frames: 64,
        wal_bytes: 1024 * 1024,
        redo_targets: 64,
        redo_bytes: 1024 * 1024,
        distinct_pages_and_extents: 64,
        operation_bindings: 64,
        staging_bytes: 4 * 1024 * 1024,
        recovery_memory_bytes: 64 * 1024 * 1024,
        dirty_frames: 64,
        concurrent_commands: 8,
        publication_effects: 4,
        cleanup_candidates: 64,
        cleanup_bytes: 1024 * 1024,
        observation_bytes: 4 * 1024 * 1024,
    })
    .unwrap();
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let authority = PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
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
}
