//! Pure topology-delta negatives. C.9 member provenance is exercised by the
//! production recovery journeys, not claimed by these direct predicate tests.

use super::*;
use worth_store_physical_format::{
    durable_artifact_checksum, DurableExtentRecordPlacement, DurableFreeSpaceManifestHeader,
    DurableInlineRecordPlacement, DurablePhysicalRootManifest, ExtentChunkCoordinate,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryRootState, PhysicalExtentId, PhysicalFreeSpaceMembershipBlock,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot,
    PhysicalRootRoutingBlock, RecordArtifactFile, RecordFrameCoordinate,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadKeyV1, SelectedRecordContentClass,
    SelectedRecordRouteMetadata,
};
use worth_store_wal::LogSequenceNumber;

fn record(number: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([4; 16], number).unwrap()
}

fn route(number: u64, extent: u64, offset: u64) -> CurrentPhysicalRecordPlacement {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(extent).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, 4096).unwrap();
    let metadata =
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(record(number), cell, 100, range, metadata)
            .unwrap(),
    )
}

fn free_entry(offset: u64, length: u64, generation: u64) -> RecordFreeSpaceManifestEntry {
    RecordFreeSpaceManifestEntry::arena_range(
        ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, length).unwrap(),
        generation,
    )
    .unwrap()
}

fn snapshot(
    generation: u64,
    routes: &[CurrentPhysicalRecordPlacement],
    free_entries: &[RecordFreeSpaceManifestEntry],
    next_extent: u64,
    latest: Option<IndexedThroughBlobPublication>,
    format: PhysicalRecordFormatDeclaration,
) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
    let free_block =
        PhysicalFreeSpaceMembershipBlock::leaf(7, generation, 1, free_entries.to_vec(), 4).unwrap();
    let free = DurableFreeSpaceManifestHeader::new(
        generation,
        7,
        4,
        4,
        free_entries.len() as u64,
        1,
        1,
        next_extent,
        2,
        20480,
        4096,
        2,
        Some(free_block.reference(durable_artifact_checksum(&free_block.encode(format)))),
    )
    .unwrap();
    let routing = PhysicalRootRoutingBlock::leaf(7, generation, 1, routes.to_vec(), 4).unwrap();
    let root = DurablePhysicalRootManifest::builder(
        generation,
        7,
        4,
        durable_artifact_checksum(&free.encode(format)),
    )
    .record_count(routes.len() as u64)
    .routing_root(Some(
        routing.reference(durable_artifact_checksum(&routing.encode(format))),
    ))
    .free_space_root(free.root())
    .next_block(2)
    .latest_blob_publication(latest)
    .admit()
    .unwrap()
    .with_maintenance_protocol();
    (root, free)
}

fn projection(projected: CurrentPhysicalRecordPlacement) -> PersistedPhysicalRecoveryProjection {
    let CurrentPhysicalRecordPlacement::Extent(extent) = projected else {
        unreachable!()
    };
    let chunk = ExtentChunkCoordinate::new(record(3), extent.extent_cell(), 100, 0, 1).unwrap();
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 1 }, 8192, 4).unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(chunk),
        coordinate,
        &[0; 4],
    )
    .unwrap();
    let root = PersistedPhysicalRecoveryRootState::new(4096, 1, 4, vec![], None, None).unwrap();
    PersistedPhysicalRecoveryProjection::new(
        1,
        root,
        vec![record(3)],
        vec![frame],
        vec![projected],
        vec![],
        vec![],
    )
    .unwrap()
}

#[test]
fn ordinary_append_delta_rejects_unrelated_route_free_and_root_hint_changes() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source_routes = [route(1, 1, 0), route(2, 2, 4096)];
    let result_routes = [route(1, 1, 0), route(2, 2, 4096), route(3, 3, 8192)];
    let source_entries = [free_entry(8192, 12288, 1)];
    let result_entries = [free_entry(12288, 8192, 2)];
    let (source_root, source_free) = snapshot(1, &source_routes, &source_entries, 3, None, format);
    let (result_root, result_free) = snapshot(2, &result_routes, &result_entries, 4, None, format);
    let source = ReleasedInventoryView::new(
        &source_root,
        &source_free,
        &source_routes,
        &[],
        &source_entries,
    );
    let result = ReleasedInventoryView::new(
        &result_root,
        &result_free,
        &result_routes,
        &[],
        &result_entries,
    );
    let projection = projection(route(3, 3, 8192));
    assert!(root::root_matches(source, result, &projection));
    assert!(delta::routes_match(source, result, &projection).unwrap());
    assert!(delta::segments_match(source, result, &projection).unwrap());
    assert!(delta::free_matches(source, result, &projection, 16).unwrap());

    let altered_routes = [route(1, 1, 0), route(2, 2, 12288), route(3, 3, 8192)];
    let altered_route_free = [free_entry(4096, 4096, 2), free_entry(16384, 4096, 2)];
    let (altered_root, altered_free) =
        snapshot(2, &altered_routes, &altered_route_free, 4, None, format);
    let route_substitution = ReleasedInventoryView::new(
        &altered_root,
        &altered_free,
        &altered_routes,
        &[],
        &altered_route_free,
    );
    assert!(transcript(route_substitution, format, 16).is_ok());
    assert!(!delta::routes_match(source, route_substitution, &projection).unwrap());

    let altered_entries = [free_entry(12288, 4096, 2)];
    let free_substitution = ReleasedInventoryView::new(
        &result_root,
        &result_free,
        &result_routes,
        &[],
        &altered_entries,
    );
    assert!(!delta::free_matches(source, free_substitution, &projection, 16).unwrap());

    let changed_latest = IndexedThroughBlobPublication::new(2, record(2), [9; 32]).unwrap();
    let (hint_root, hint_free) = snapshot(
        2,
        &result_routes,
        &result_entries,
        4,
        Some(changed_latest),
        format,
    );
    let hint_substitution =
        ReleasedInventoryView::new(&hint_root, &hint_free, &result_routes, &[], &result_entries);
    assert!(!root::root_matches(source, hint_substitution, &projection));

    let key = ReleaseCustodyHeadKeyV1::new([7; 16], 1).unwrap();
    let head = ReleaseCustodyHeadBlockReferenceV1::new(2, 1, 0, key, key, [8; 32]).unwrap();
    let changed_head = DurablePhysicalRootManifest::builder(
        2,
        7,
        4,
        durable_artifact_checksum(&result_free.encode(format)),
    )
    .next_block(2)
    .release_custody_head_root(Some(head))
    .next_release_custody_head_block(2)
    .admit()
    .unwrap()
    .with_maintenance_protocol();
    let changed_head_result = ReleasedInventoryView::new(
        &changed_head,
        &result_free,
        &result_routes,
        &[],
        &result_entries,
    );
    assert!(!root::root_matches(
        source,
        changed_head_result,
        &projection
    ));

    let changed_frontier = DurablePhysicalRootManifest::builder(
        2,
        7,
        4,
        durable_artifact_checksum(&result_free.encode(format)),
    )
    .next_block(2)
    .next_release_custody_head_block(2)
    .admit()
    .unwrap()
    .with_maintenance_protocol();
    let changed_frontier_result = ReleasedInventoryView::new(
        &changed_frontier,
        &result_free,
        &result_routes,
        &[],
        &result_entries,
    );
    assert!(!root::root_matches(
        source,
        changed_frontier_result,
        &projection
    ));
}

#[test]
fn store_recheck_requires_exact_member_and_independent_topology() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source_routes = [route(1, 1, 0), route(2, 2, 4096)];
    let result_routes = [route(1, 1, 0), route(2, 2, 4096), route(3, 3, 8192)];
    let source_entries = [free_entry(8192, 12288, 1)];
    let result_entries = [free_entry(12288, 8192, 2)];
    let (source_root, source_free) = snapshot(1, &source_routes, &source_entries, 3, None, format);
    let (result_root, result_free) = snapshot(2, &result_routes, &result_entries, 4, None, format);
    let source = ReleasedInventoryView::new(
        &source_root,
        &source_free,
        &source_routes,
        &[],
        &source_entries,
    );
    let result = ReleasedInventoryView::new(
        &result_root,
        &result_free,
        &result_routes,
        &[],
        &result_entries,
    );
    let projection = projection(route(3, 3, 8192));
    let group = PhysicalRedoGroupBinding::new([1; 32], [2; 32], 1, 1, [3; 32]).unwrap();
    let range = WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap();
    // Only the inner delta is constructed directly here; the public constructor
    // requires an admitted C.9 member, which real recovery supplies.
    let checked = VerifiedOrdinaryRootStep::check(
        source,
        result,
        Some(range),
        [4; 32],
        group,
        RecoveryOperationFate::Indeterminate,
        [5; 32],
        &projection,
        format,
        16,
        1 << 20,
    )
    .unwrap();
    let observed = ObservedOrdinaryRootMember {
        operation: [4; 32],
        group,
        fate: RecoveryOperationFate::Indeterminate,
        canonical_redo_sha256: [5; 32],
        lsn_range: range,
        projection: &projection,
    };
    assert!(checked
        .recheck_actual_media(source, result, observed, format, 16, 1 << 20)
        .is_ok());
    // Store's admitted ceiling can greatly exceed this actual rooted
    // inventory; the ceiling must cap entries, not become a scratch charge.
    assert!(checked
        .recheck_actual_media(source, result, observed, format, 1 << 20, 1 << 20)
        .is_ok());
    let wrong_member = ObservedOrdinaryRootMember {
        canonical_redo_sha256: [6; 32],
        ..observed
    };
    assert_eq!(
        checked.recheck_actual_media(source, result, wrong_member, format, 16, 1 << 20),
        Err(OrdinaryRootStepDenial::InvalidMember)
    );
    let altered_routes = [route(1, 1, 0), route(2, 2, 12288), route(3, 3, 8192)];
    let altered_entries = [free_entry(4096, 4096, 2), free_entry(16384, 4096, 2)];
    let (altered_root, altered_free) =
        snapshot(2, &altered_routes, &altered_entries, 4, None, format);
    let altered = ReleasedInventoryView::new(
        &altered_root,
        &altered_free,
        &altered_routes,
        &[],
        &altered_entries,
    );
    assert!(checked
        .recheck_actual_media(source, altered, observed, format, 16, 1 << 20)
        .is_err());
}

#[test]
fn retired_inline_witness_may_advance_page_generation_but_not_slot_or_content_identity() {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap();
    let page = PhysicalPageId::from_raw(1).unwrap();
    let slot = PhysicalRecordSlot::from_raw(6).unwrap();
    let make = |generation: u64, slot_generation: u64, payload_bytes: u64| {
        let generation = PhysicalGeneration::from_raw(generation).unwrap();
        let slot_generation = PhysicalGeneration::from_raw(slot_generation).unwrap();
        DurableInlineRecordPlacement::new_selected(
            record(1),
            authority
                .segment_cell(segment)
                .with_segment_generation(generation),
            authority
                .page_cell(segment, page)
                .with_page_generation(generation),
            authority
                .slot_cell(segment, page, slot)
                .with_slot_generation(slot_generation),
            407,
            payload_bytes,
            SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::BTreeNode {
                family_code: 2,
            })
            .unwrap(),
        )
        .unwrap()
    };
    let source = CurrentPhysicalRecordPlacement::Inline(make(7, 1, 232));
    assert!(delta::retired_witness_matches(source, make(8, 1, 232)));
    assert!(!delta::retired_witness_matches(source, make(6, 1, 232)));
    assert!(!delta::retired_witness_matches(source, make(8, 2, 232)));
    assert!(!delta::retired_witness_matches(source, make(8, 1, 233)));
}
