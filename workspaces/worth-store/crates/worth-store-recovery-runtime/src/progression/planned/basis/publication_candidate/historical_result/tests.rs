use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, ExtentArenaId, ExtentArenaRange,
    IndexedThroughBlobPublication, ManifestBlockReference, PersistedRecordIdentity,
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry, SelectedRecordContentClass,
    SelectedRecordRouteMetadata,
};

use super::*;

struct Fixture {
    source_root: DurablePhysicalRootManifest,
    source: RecoverySelectedSourceInventory,
    source_routes: [CurrentPhysicalRecordPlacement; 2],
    result_root: DurablePhysicalRootManifest,
    result: RecoverySelectedSourceInventory,
    result_routes: [CurrentPhysicalRecordPlacement; 2],
    dropped: [PersistedRecordIdentity; 1],
    projected: [CurrentPhysicalRecordPlacement; 1],
}

impl Fixture {
    fn matches(&self) -> bool {
        verified_historical_release_transition(
            &self.source_root,
            &self.source,
            &self.source_routes,
            &self.result_root,
            &self.result,
            &self.result_routes,
            &self.dropped,
            &self.projected,
            None,
            None,
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            16,
            1 << 20,
        )
        .is_some()
    }
}

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn route(
    record_id: u64,
    extent_id: u64,
    arena_id: u64,
    offset: u64,
    length: u64,
) -> CurrentPhysicalRecordPlacement {
    let extent = PhysicalExtentId::from_raw(extent_id).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(extent)
        .with_extent_generation(generation);
    let range =
        ExtentArenaRange::new(ExtentArenaId::new(arena_id).unwrap(), offset, length).unwrap();
    let metadata =
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(record(record_id), cell, 100, range, metadata)
            .unwrap(),
    )
}

fn free_header(
    generation: u64,
    next_extent: u64,
    next_arena: u64,
) -> DurableFreeSpaceManifestHeader {
    DurableFreeSpaceManifestHeader::new(
        generation,
        7,
        4,
        16,
        0,
        1,
        1,
        next_extent,
        next_arena,
        8192,
        4096,
        1,
        None,
    )
    .unwrap()
}

fn inventory(free_space: DurableFreeSpaceManifestHeader) -> RecoverySelectedSourceInventory {
    RecoverySelectedSourceInventory {
        free_space,
        segment_pages: BTreeMap::new(),
        segment_topology: BTreeMap::new(),
        free_entries: Box::new([]),
        free_topology: BTreeMap::new(),
        source_artifacts: Box::new([]),
    }
}

fn root(
    generation: u64,
    free: &DurableFreeSpaceManifestHeader,
    first: u64,
    last: u64,
    latest: Option<IndexedThroughBlobPublication>,
    maintenance: bool,
) -> DurablePhysicalRootManifest {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let reference =
        ManifestBlockReference::new(generation, 1, 0, 99, record(first), record(last)).unwrap();
    let root = DurablePhysicalRootManifest::builder(
        generation,
        7,
        4,
        durable_artifact_checksum(&free.encode(format)),
    )
    .record_count(2)
    .next_block(2)
    .routing_root(Some(reference))
    .latest_blob_publication(latest)
    .admit()
    .unwrap();
    if maintenance {
        root.with_maintenance_protocol()
    } else {
        root
    }
}

fn fixture() -> Fixture {
    let source_free = free_header(1, 3, 2);
    let result_free = free_header(2, 4, 3);
    let latest = IndexedThroughBlobPublication::new(1, record(1), [1; 32]).unwrap();
    let source_root = root(1, &source_free, 1, 2, Some(latest), false);
    let result_root = root(2, &result_free, 2, 3, None, true);
    let survivor = route(2, 2, 1, 4096, 4096);
    let projected = route(3, 3, 2, 0, 8192);
    Fixture {
        source_root,
        source: inventory(source_free),
        source_routes: [route(1, 1, 1, 0, 4096), survivor],
        result_root,
        result: inventory(result_free),
        result_routes: [survivor, projected],
        dropped: [record(1)],
        projected: [projected],
    }
}

#[test]
fn exact_historical_result_rejects_same_count_route_and_free_substitutions() {
    let mut honest = fixture();
    assert!(honest.matches());
    honest.result_routes[0] = route(2, 4, 1, 4096, 4096);
    assert!(!honest.matches(), "same-count unrelated route substitution");
    let mut wrong_free = fixture();
    wrong_free.result.free_entries = Box::new([RecordFreeSpaceManifestEntry::arena_range(
        ExtentArenaRange::new(ExtentArenaId::new(3).unwrap(), 0, 4096).unwrap(),
        2,
    )
    .unwrap()]);
    assert!(!wrong_free.matches(), "invented free arena range");
}

#[test]
fn exact_historical_result_requires_maintenance_and_drop_filtered_root_hints() {
    let mut wrong_maintenance = fixture();
    wrong_maintenance.result_root =
        root(2, &wrong_maintenance.result.free_space, 2, 3, None, false);
    assert!(!wrong_maintenance.matches());
    let mut wrong_hint = fixture();
    let latest = IndexedThroughBlobPublication::new(2, record(2), [2; 32]).unwrap();
    wrong_hint.result_root = root(2, &wrong_hint.result.free_space, 2, 3, Some(latest), true);
    assert!(!wrong_hint.matches());
}
