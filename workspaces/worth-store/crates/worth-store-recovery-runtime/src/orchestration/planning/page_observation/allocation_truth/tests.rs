use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::recovery_wal::{LogSequenceNumber, WalLsnRange};
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest,
};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement, PersistedInlineSegmentAllocation,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordSlot, PhysicalSegmentId, RecordAllocationClass, RecordArtifactFile,
    RecordFrameCoordinate, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{decode_physical_redo_records, PhysicalRedoTarget};
use worth_store_test_support::harness::physical_residency::{
    canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld,
};

use super::{allocation_sequence_above, reusable_capacity};
use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits,
    PhysicalRecoveryOpenRequest, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration,
};

#[path = "tests/capacity_tests.rs"]
mod capacity_tests;

#[test]
fn committed_frontiers_allow_reserved_gaps_but_not_reused_or_unordered_ids() {
    assert!(allocation_sequence_above([7, 8, 9], 7));
    assert!(allocation_sequence_above([7, 9], 7));
    assert!(allocation_sequence_above([8], 7));
    assert!(!allocation_sequence_above([6], 7));
    assert!(!allocation_sequence_above([7, 7], 7));
    assert!(!allocation_sequence_above([9, 8], 7));
}

#[test]
fn reusable_capacity_requires_selected_free_space_truth() {
    let exact =
        RecordFreeSpaceManifestEntry::new(RecordAllocationClass::InlinePage, 3, 3, 2, 8).unwrap();
    assert_eq!(reusable_capacity(exact, 8, 2, 4), Some(2));
    assert_eq!(reusable_capacity(exact, 7, 2, 4), None);
    assert_eq!(reusable_capacity(exact, 8, 1, 4), None);
    assert_eq!(reusable_capacity(exact, 8, 2, 5), None);
}

#[test]
fn selected_capacity_rejects_bypassing_or_prematurely_spilling_reusable_pages() {
    let lawful = selected_world("allocation-reuse-lawful", 4);
    let first_page = next_page(&lawful.placements);
    let reused = target(1, first_page, 1, 1, 2, 8);
    assert_admitted(lawful, vec![reused]);

    let one_page = selected_world("allocation-reuse-first", 4);
    let first_page = next_page(&one_page.placements);
    let no_reuse = target(2, first_page, 1, 2, 1, 1);
    assert_rejected(one_page, vec![no_reuse]);

    let premature = selected_world("allocation-premature-spill", 4);
    let first_page = next_page(&premature.placements);
    let reuse = target(1, first_page, 1, 1, 2, 2);
    let spill = target(2, first_page + 1, 1, 2, 1, 3);
    assert_rejected(premature, vec![reuse, spill]);

    let reordered = selected_world("allocation-reordered-spill", 2);
    let first_page = next_page(&reordered.placements);
    let spill_first = target(2, first_page, 1, 2, 1, 4);
    let reuse_late = target(1, first_page + 1, 1, 1, 2, 5);
    assert_rejected(reordered, vec![spill_first, reuse_late]);
}

#[test]
fn selected_source_inventory_denies_before_crossing_manifest_block_read() {
    let world = selected_world("allocation-cumulative-entry-budget", 4);
    assert!(!world.placements.is_empty());
    let SelectedWorld {
        authority,
        coordination,
        root,
        placements: _,
        retained,
    } = world;
    let AdmittedPlatformAuthority {
        media,
        session,
        _world_binding,
        ..
    } = authority;
    let mut discovery = media.bounded_discovery(64, 1024 * 1024).unwrap();
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let (result, _) = crate::orchestration::planning::selected_source_inventory::observe(
        &mut discovery,
        &root,
        format,
        1,
        1024 * 1024,
    );
    assert_eq!(
        result,
        Err(super::PageObservationFailure::ManifestEntryLimit)
    );
    assert_eq!(discovery.counters().addressed_artifacts_read, 2);
    drop(discovery.finish());
    assert!(coordination.shutdown_is_quiescent());
    session.refuse();
    drop(retained);
}

struct SelectedWorld {
    authority: AdmittedPlatformAuthority,
    coordination: crate::orchestration::RecoveryCoordination,
    root: worth_store_physical_format::DurablePhysicalRootManifest,
    placements: Vec<CurrentPhysicalRecordPlacement>,
    retained: worth_store_test_support::TemporaryDirectory,
}

fn selected_world(name: &str, segment_pages: u32) -> SelectedWorld {
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
        panic!("allocation-truth checkpoint admission")
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

fn admitted_recovery(root: &std::path::Path) -> crate::AdmittedPhysicalRecovery {
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

fn assert_rejected(world: SelectedWorld, targets: Vec<PhysicalRedoTarget>) {
    assert_result(world, targets, false);
}

fn assert_admitted(world: SelectedWorld, targets: Vec<PhysicalRedoTarget>) {
    assert_result(world, targets, true);
}

fn assert_result(world: SelectedWorld, targets: Vec<PhysicalRedoTarget>, expected: bool) {
    let SelectedWorld {
        authority,
        coordination,
        root,
        placements,
        retained,
    } = world;
    let AdmittedPlatformAuthority {
        media,
        session,
        _world_binding,
        ..
    } = authority;
    let mut discovery = media.bounded_discovery(64, 1024 * 1024).unwrap();
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let (selected_source, integrity_trace) =
        crate::orchestration::planning::selected_source_inventory::observe(
            &mut discovery,
            &root,
            format,
            64,
            1024 * 1024,
        );
    let selected_source = selected_source.unwrap();
    assert_eq!(
        integrity_trace.observations().len() as u64,
        integrity_trace.counters().attempted,
        "every root-tree/free-space ingress attempt must retain its scoped observation"
    );
    assert_eq!(integrity_trace.counters().attempted, 3);
    assert_eq!(integrity_trace.counters().admitted, 3);
    let families = integrity_trace
        .observations()
        .iter()
        .map(|observation| observation.scope().artifact_family())
        .collect::<Vec<_>>();
    assert_eq!(
        families,
        vec![
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        ]
    );
    // This row proves selected capacity geometry. Exact admitted WAL membership
    // is exercised separately at the page-observation entry, not by these descriptors.
    let refs = targets.iter().collect::<Vec<_>>();
    let result = super::admit_inline_allocations(
        &root,
        &placements,
        &refs,
        &selected_source.free_space,
        &selected_source.free_entries,
    )
    .and_then(|()| super::admit_extent_allocations(&refs, &selected_source.free_space));
    if expected {
        assert_eq!(result, Ok(()));
    } else {
        assert!(
            matches!(
                &result,
                Err(super::PageObservationFailure::InvalidTarget(_))
            ),
            "unexpected allocation admission result: {result:?}"
        );
    }
    drop(discovery.finish());
    assert!(coordination.shutdown_is_quiescent());
    session.refuse();
    drop(retained);
}

fn next_page(placements: &[CurrentPhysicalRecordPlacement]) -> u64 {
    placements
        .iter()
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline) => Some(inline.page().get()),
            CurrentPhysicalRecordPlacement::Extent(_) => None,
        })
        .max()
        .unwrap()
        + 1
}

fn target(
    segment: u64,
    page: u64,
    page_generation: u64,
    artifact_segment: u64,
    artifact_generation: u64,
    ordinal: u64,
) -> PhysicalRedoTarget {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment_id = PhysicalSegmentId::from_raw(segment).unwrap();
    let page_cell = authority
        .page_cell(segment_id, PhysicalPageId::from_raw(page).unwrap())
        .with_page_generation(PhysicalGeneration::from_raw(page_generation).unwrap());
    let bytes = vec![ordinal as u8; 8];
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: artifact_segment,
            generation: artifact_generation,
        },
        0,
        bytes.len() as u32,
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(page_cell),
        coordinate,
        &bytes,
    )
    .unwrap();
    let record = PersistedRecordIdentity::new([ordinal as u8; 16], ordinal).unwrap();
    let slot = authority
        .slot_cell(
            segment_id,
            page_cell.page_id(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
    let artifact_cell = authority
        .segment_cell(PhysicalSegmentId::from_raw(artifact_segment).unwrap())
        .with_segment_generation(PhysicalGeneration::from_raw(artifact_generation).unwrap());
    let placement =
        DurableInlineRecordPlacement::new(record, artifact_cell, page_cell, slot, 4, 4).unwrap();
    let routing = RecordSegmentPageManifestEntry::new(page_cell, artifact_cell, 1, 0).unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new(
        1,
        PersistedPhysicalRecoveryRootState::new(
            4096,
            1,
            4,
            vec![PersistedInlineSegmentAllocation::new(artifact_cell, 4, 1).unwrap()],
            Some(record),
            Some(artifact_cell),
        )
        .unwrap(),
        vec![record],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Inline(placement)],
        vec![routing],
        Vec::new(),
    )
    .unwrap();
    let mut target = Vec::new();
    target.push(1);
    target.extend_from_slice(&segment.to_le_bytes());
    target.extend_from_slice(&page.to_le_bytes());
    target.extend_from_slice(&page_generation.to_le_bytes());
    target.push(5);
    target.extend_from_slice(&artifact_segment.to_le_bytes());
    target.extend_from_slice(&artifact_generation.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    let mut encoded = Vec::new();
    field(&mut encoded, b"store.physical.wal.canonical-redo.v3");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u32.to_le_bytes());
    encoded.extend_from_slice(&10_u64.to_le_bytes());
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, &target);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    encoded.extend_from_slice(&digest);
    field(&mut encoded, b"redo");
    field(&mut encoded, &projection.encode());
    decode_physical_redo_records(
        &encoded,
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap(),
        1,
    )
    .unwrap()[0]
        .targets()[0]
        .clone()
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}
