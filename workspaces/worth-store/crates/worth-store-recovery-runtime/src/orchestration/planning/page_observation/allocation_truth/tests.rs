use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, RecordFreeSpaceManifestEntry,
};
use worth_store_recovery_physics::PhysicalRedoTarget;

use super::super::inline_image_fixture::inline_image;
use super::{first_invalid_extent, reusable_capacity};
use crate::orchestration::planning::selected_source_inventory;
use crate::orchestration::planning::selected_world_fixture::{selected_world, SelectedWorld};

#[path = "tests/capacity_tests.rs"]
mod capacity_tests;

#[test]
fn committed_frontiers_allow_reserved_gaps_but_not_reused_or_unordered_ids() {
    assert_eq!(first_invalid_extent([7, 8, 9], 7), None);
    assert_eq!(first_invalid_extent([7, 9], 7), None);
    assert_eq!(first_invalid_extent([8], 7), None);
    assert_eq!(first_invalid_extent([6], 7), Some(6));
    assert_eq!(first_invalid_extent([7, 7], 7), Some(7));
    assert_eq!(first_invalid_extent([9, 8], 7), Some(8));
}

#[test]
fn reusable_capacity_requires_selected_free_space_truth() {
    let exact = RecordFreeSpaceManifestEntry::inline_frontier(3, 3, 2, 8).unwrap();
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
fn selected_source_inventory_is_charged_its_root_and_leaf_entries_and_no_block() {
    let observe = |name: &str, entries: u64| {
        selected_world(name, 4).read(|source| {
            selected_source_inventory::observe(
                source.discovery,
                source.root,
                source.format,
                entries,
            )
            .0
        })
    };
    let inventory = observe("allocation-entry-charge-whole", 64).unwrap();
    let need = 1 + (inventory.segment_pages.len() + inventory.free_entries.len()) as u64;
    assert!(need > 2, "the world holds more than one leaf entry");
    assert!(observe("allocation-entry-charge-need", need).is_ok());
    assert_eq!(
        observe("allocation-entry-charge-short", need - 1).err(),
        Some(super::PageObservationFailure::ManifestEntryLimit)
    );
}

fn assert_rejected(world: SelectedWorld, targets: Vec<PhysicalRedoTarget>) {
    assert_result(world, targets, false);
}

fn assert_admitted(world: SelectedWorld, targets: Vec<PhysicalRedoTarget>) {
    assert_result(world, targets, true);
}

fn assert_result(world: SelectedWorld, targets: Vec<PhysicalRedoTarget>, expected: bool) {
    let (selected_source, integrity_trace, root, placements) = world.read(|source| {
        let (selected_source, integrity_trace) =
            selected_source_inventory::observe(source.discovery, source.root, source.format, 64);
        (
            selected_source.unwrap(),
            integrity_trace,
            source.root.clone(),
            source.placements.to_vec(),
        )
    });
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
    let record = PersistedRecordIdentity::new([ordinal as u8; 16], ordinal).unwrap();
    inline_image(
        (segment, page, page_generation),
        (artifact_segment, artifact_generation),
        &[record],
        ordinal as u8,
    )
    .1
}
