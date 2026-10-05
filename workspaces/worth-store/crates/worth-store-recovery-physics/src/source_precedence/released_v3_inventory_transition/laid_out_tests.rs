use worth_store_physical_format::{
    IndexedThroughBlobPublication, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalSegmentId,
};

use super::super::tests::{free_entry, record, route, snapshot};
use super::*;
use crate::source_precedence::{entries_past, scratch_past};

fn segment() -> RecordSegmentPageManifestEntry {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let id = PhysicalSegmentId::from_raw(1).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let cell = authority
        .segment_cell(id)
        .with_segment_generation(generation);
    let page = authority
        .page_cell(id, PhysicalPageId::from_raw(2).unwrap())
        .with_page_generation(generation);
    RecordSegmentPageManifestEntry::new(page, cell, 1, 0).unwrap()
}

struct Sides {
    source: Snapshot,
    result: Snapshot,
    source_routes: [CurrentPhysicalRecordPlacement; 2],
    result_routes: [CurrentPhysicalRecordPlacement; 2],
    source_free: [RecordFreeSpaceManifestEntry; 1],
    result_free: [RecordFreeSpaceManifestEntry; 1],
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn sides() -> Sides {
    let source_routes = [route(1, 1, 0), route(2, 2, 4096)];
    let result_routes = [route(2, 2, 4096), route(3, 3, 8192)];
    let source_free = [free_entry(8192, 12288, 1)];
    let result_free = [free_entry(12288, 8192, 2)];
    let latest = IndexedThroughBlobPublication::new(1, record(1), [1; 32]).unwrap();
    Sides {
        source: snapshot(
            1,
            &source_routes,
            source_free[0],
            3,
            Some(latest),
            false,
            format(),
        ),
        result: snapshot(2, &result_routes, result_free[0], 4, None, true, format()),
        source_routes,
        result_routes,
        source_free,
        result_free,
    }
}

type Snapshot = (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader);

fn parts<'a>(
    (root, free): &'a Snapshot,
    routes: &'a [CurrentPhysicalRecordPlacement],
    free_entries: &'a [RecordFreeSpaceManifestEntry],
    segments: &'a [RecordSegmentPageManifestEntry],
) -> ReleasedInventoryParts<'a, impl ExactSizeIterator<Item = RecordSegmentPageManifestEntry> + 'a>
{
    ReleasedInventoryParts {
        root,
        free,
        routes,
        segments: segments.iter().copied(),
        free_entries,
    }
}

fn view<'a>(
    (root, free): &'a Snapshot,
    routes: &'a [CurrentPhysicalRecordPlacement],
    free_entries: &'a [RecordFreeSpaceManifestEntry],
) -> ReleasedInventoryView<'a> {
    ReleasedInventoryView::new(root, free, routes, &[], free_entries)
}

impl Sides {
    /// The transition with `segments` on both sides, checked within `scratch`.
    fn laid_out(
        &self,
        segments: &[RecordSegmentPageManifestEntry],
        dropped: &[PersistedRecordIdentity],
        entries: u64,
        scratch: u64,
    ) -> Result<(VerifiedReleasedV3InventoryTransition, u64), Denial> {
        VerifiedReleasedV3InventoryTransition::admit_laid_out(
            parts(
                &self.source,
                &self.source_routes,
                &self.source_free,
                segments,
            ),
            parts(
                &self.result,
                &self.result_routes,
                &self.result_free,
                segments,
            ),
            dropped,
            &[self.result_routes[1]],
            None,
            None,
            format(),
            entries,
            scratch,
        )
    }

    /// What the check alone holds at its peak, segments aside.
    fn check_peak(&self) -> u64 {
        VerifiedReleasedV3InventoryTransition::maximum_construction_heap_bytes(
            view(&self.source, &self.source_routes, &self.source_free),
            view(&self.result, &self.result_routes, &self.result_free),
            &[self.result_routes[1]],
            16,
        )
        .unwrap()
    }
}

#[test]
fn a_laid_out_transition_is_the_slice_transition_holding_no_segments() {
    let sides = sides();
    let (transition, held) = sides.laid_out(&[], &[record(1)], 16, 1 << 20).unwrap();
    let sliced = VerifiedReleasedV3InventoryTransition::admit(
        view(&sides.source, &sides.source_routes, &sides.source_free),
        view(&sides.result, &sides.result_routes, &sides.result_free),
        &[record(1)],
        &[sides.result_routes[1]],
        format(),
        16,
        1 << 20,
        None,
    );
    assert_eq!((Ok(transition), held), (sliced, 0));
}

#[test]
fn a_check_past_what_the_segments_leave_names_both_against_the_whole_ceiling() {
    let sides = sides();
    let held = 2 * SEGMENT_WIDTH;
    let peak = sides.check_peak();
    let ceiling = held + peak - 1;
    assert_eq!(
        sides
            .laid_out(&[segment()], &[record(1)], 16, ceiling)
            .map(drop),
        Err(Denial::BoundExceeded(scratch_past(held + peak, ceiling)))
    );
}

#[test]
fn segments_past_the_ceiling_are_refused_before_any_is_laid_out() {
    let sides = sides();
    let held = 2 * SEGMENT_WIDTH;
    assert_eq!(
        sides
            .laid_out(&[segment()], &[record(1)], 16, held - 1)
            .map(drop),
        Err(Denial::BoundExceeded(scratch_past(held, held - 1)))
    );
}

#[test]
fn entries_and_refusals_that_are_no_limit_stay_as_the_check_named_them() {
    let sides = sides();
    // Each view holds two routes: one entry admits neither, whatever is held.
    assert_eq!(
        sides
            .laid_out(&[segment()], &[record(1)], 1, 1 << 20)
            .map(drop),
        Err(Denial::BoundExceeded(entries_past(2, 1)))
    );
    // Nothing dropped is no legal release.
    assert_eq!(
        sides.laid_out(&[], &[], 16, 1 << 20).map(drop),
        Err(Denial::InvalidDelta)
    );
}
