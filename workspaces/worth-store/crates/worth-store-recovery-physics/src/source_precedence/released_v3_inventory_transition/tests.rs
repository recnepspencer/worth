use super::*;
use crate::ExceededRootHistoryBound as Exceeded;
use worth_store_physical_format::{
    durable_artifact_checksum, DurableExtentRecordPlacement, ExtentArenaId,
    IndexedThroughBlobPublication, PhysicalExtentId, PhysicalFreeSpaceMembershipBlock,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRootRoutingBlock,
    SelectedRecordContentClass, SelectedRecordRouteMetadata,
};
use ReleasedV3InventoryTransitionDenial as Denial;

pub(super) fn record(number: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([4; 16], number).unwrap()
}

pub(super) fn route(number: u64, extent: u64, offset: u64) -> CurrentPhysicalRecordPlacement {
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

pub(super) fn free_entry(
    offset: u64,
    length: u64,
    generation: u64,
) -> RecordFreeSpaceManifestEntry {
    RecordFreeSpaceManifestEntry::arena_range(
        ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, length).unwrap(),
        generation,
    )
    .unwrap()
}

pub(super) fn snapshot(
    generation: u64,
    routes: &[CurrentPhysicalRecordPlacement],
    free_entry: RecordFreeSpaceManifestEntry,
    next_extent: u64,
    latest: Option<IndexedThroughBlobPublication>,
    maintenance: bool,
    format: PhysicalRecordFormatDeclaration,
) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
    snapshot_with_free_entries(
        generation,
        routes,
        &[free_entry],
        next_extent,
        latest,
        maintenance,
        format,
        20480,
    )
}

fn snapshot_with_free_entries(
    generation: u64,
    routes: &[CurrentPhysicalRecordPlacement],
    free_entries: &[RecordFreeSpaceManifestEntry],
    next_extent: u64,
    latest: Option<IndexedThroughBlobPublication>,
    maintenance: bool,
    format: PhysicalRecordFormatDeclaration,
    arena_capacity: u64,
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
        arena_capacity,
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
    .unwrap();
    (
        if maintenance {
            root.with_maintenance_protocol()
        } else {
            root
        },
        free,
    )
}

#[test]
fn checked_transition_rejects_self_consistent_same_count_route_and_free_forgery() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source_routes = [route(1, 1, 0), route(2, 2, 4096)];
    let result_routes = [route(2, 2, 4096), route(3, 3, 8192)];
    let projected = [route(3, 3, 8192)];
    let dropped = [record(1)];
    let source_free_entries = [free_entry(8192, 12288, 1)];
    let result_free_entries = [free_entry(12288, 8192, 2)];
    let latest = IndexedThroughBlobPublication::new(1, record(1), [1; 32]).unwrap();
    let (source_root, source_free) = snapshot(
        1,
        &source_routes,
        source_free_entries[0],
        3,
        Some(latest),
        false,
        format,
    );
    let (result_root, result_free) = snapshot(
        2,
        &result_routes,
        result_free_entries[0],
        4,
        None,
        true,
        format,
    );
    let source = ReleasedInventoryView::new(
        &source_root,
        &source_free,
        &source_routes,
        &[],
        &source_free_entries,
    );
    let result = ReleasedInventoryView::new(
        &result_root,
        &result_free,
        &result_routes,
        &[],
        &result_free_entries,
    );
    let honest = VerifiedReleasedV3InventoryTransition::admit(
        source,
        result,
        &dropped,
        &projected,
        format,
        16,
        1 << 20,
        None,
    )
    .expect("canonical release transition");
    assert_ne!(honest.source_topology(), honest.result_topology());
    let exact_peak = VerifiedReleasedV3InventoryTransition::maximum_construction_heap_bytes(
        source, result, &projected, 16,
    )
    .unwrap();
    assert_eq!(
        VerifiedReleasedV3InventoryTransition::admit(
            source,
            result,
            &dropped,
            &projected,
            format,
            16,
            exact_peak - 1,
            None,
        ),
        Err(Denial::BoundExceeded(Exceeded::scratch(
            exact_peak,
            exact_peak - 1
        )))
    );
    // Each view holds two routes: one entry admits neither.
    let narrow = VerifiedReleasedV3InventoryTransition::admit(
        source,
        result,
        &dropped,
        &projected,
        format,
        1,
        1 << 20,
        None,
    );
    let past_entries = Denial::BoundExceeded(Exceeded::entries(2, 1));
    assert_eq!(narrow, Err(past_entries.clone()));
    assert_eq!(past_entries.exceeded_bound(), Some(Exceeded::entries(2, 1)));
    assert_eq!(Denial::InvalidDelta.exceeded_bound(), None);
    let exact = VerifiedReleasedV3InventoryTransition::admit(
        source, result, &dropped, &projected, format, 16, exact_peak, None,
    )
    .unwrap();
    assert_eq!(exact, honest);
    assert_eq!(exact.scratch_bytes(), exact_peak);
    assert_eq!(
        exact.owned_heap_bytes(),
        Some(std::mem::size_of_val(&projected) as u64)
    );

    // A different, still allocated/nonfree route is a stronger adversary than
    // an overlap with the selected free range: only the legal delta rejects it.
    let altered_routes = [route(2, 2, 0), route(3, 3, 8192)];
    let (altered_root, altered_free) = snapshot(
        2,
        &altered_routes,
        result_free_entries[0],
        4,
        None,
        true,
        format,
    );
    let altered = ReleasedInventoryView::new(
        &altered_root,
        &altered_free,
        &altered_routes,
        &[],
        &result_free_entries,
    );
    assert_eq!(
        VerifiedReleasedV3InventoryTransition::admit(
            source,
            altered,
            &dropped,
            &projected,
            format,
            16,
            1 << 20,
            None,
        ),
        Err(ReleasedV3InventoryTransitionDenial::InvalidDelta),
    );

    let altered_free_entries = [free_entry(12288, 4096, 2)];
    let (altered_root, altered_free) = snapshot(
        2,
        &result_routes,
        altered_free_entries[0],
        4,
        None,
        true,
        format,
    );
    let altered = ReleasedInventoryView::new(
        &altered_root,
        &altered_free,
        &result_routes,
        &[],
        &altered_free_entries,
    );
    assert_eq!(
        VerifiedReleasedV3InventoryTransition::admit(
            source,
            altered,
            &dropped,
            &projected,
            format,
            16,
            1 << 20,
            None,
        ),
        Err(ReleasedV3InventoryTransitionDenial::InvalidDelta),
    );
}

#[test]
fn streamed_subtraction_preserves_untouched_generation_and_rejects_overlapping_ranges() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source_routes = [route(1, 1, 0), route(2, 2, 12288)];
    let projected = [route(3, 3, 16384), route(4, 4, 20480)];
    let result_routes = [source_routes[1], projected[0], projected[1]];
    let source_entries = [free_entry(8192, 4096, 1), free_entry(16384, 8192, 1)];
    let result_entries = [source_entries[0]];
    let latest = IndexedThroughBlobPublication::new(1, record(1), [1; 32]).unwrap();
    let (source_root, source_free) = snapshot_with_free_entries(
        1,
        &source_routes,
        &source_entries,
        3,
        Some(latest),
        false,
        format,
        24576,
    );
    let (result_root, result_free) = snapshot_with_free_entries(
        2,
        &result_routes,
        &result_entries,
        5,
        None,
        true,
        format,
        24576,
    );
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
    let admitted = VerifiedReleasedV3InventoryTransition::admit(
        source,
        result,
        &[record(1)],
        &projected,
        format,
        16,
        1 << 20,
        None,
    )
    .unwrap();
    assert_eq!(admitted.projected(), &projected);
    let restamped = [free_entry(8192, 4096, 2)];
    let (root, free) =
        snapshot_with_free_entries(2, &result_routes, &restamped, 5, None, true, format, 24576);
    let altered = ReleasedInventoryView::new(&root, &free, &result_routes, &[], &restamped);
    assert_eq!(
        VerifiedReleasedV3InventoryTransition::admit(
            source,
            altered,
            &[record(1)],
            &projected,
            format,
            16,
            1 << 20,
            None,
        ),
        Err(ReleasedV3InventoryTransitionDenial::InvalidDelta)
    );
    let overlap = [projected[0], route(4, 4, 16384)];
    assert_eq!(
        delta::validate_arenas(source, result, &overlap, 16, 1 << 20),
        Err(ReleasedV3InventoryTransitionDenial::InvalidDelta)
    );
}

#[test]
fn actual_reservation_failure_retains_its_typed_allocator_cause() {
    let denial = reserve::<u8>(usize::MAX, 0, u64::MAX).unwrap_err();
    let ReleasedV3InventoryTransitionDenial::Allocation {
        requested_bytes,
        cause,
    } = denial
    else {
        panic!("capacity rejection is not a declared budget crossing");
    };
    assert_eq!(requested_bytes, usize::MAX as u64);
    assert_eq!(
        cause,
        Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err()
    );
}
