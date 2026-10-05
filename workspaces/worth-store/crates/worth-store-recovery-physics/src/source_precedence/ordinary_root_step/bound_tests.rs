//! A step past an admitted bound says which bound and what it needed.

use super::*;
use crate::ExceededRootHistoryBound as Exceeded;

fn far_route(arena: u64) -> CurrentPhysicalRecordPlacement {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(arena).unwrap(), 0, 4096).unwrap();
    let metadata =
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(record(3), cell, 100, range, metadata).unwrap(),
    )
}

#[test]
fn a_step_past_its_entries_or_scratch_names_the_bound_and_the_need() {
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
    let check = |entries, scratch| {
        VerifiedOrdinaryRootStep::check(
            source,
            result,
            None,
            [4; 32],
            group,
            RecoveryOperationFate::Indeterminate,
            [5; 32],
            &projection,
            format,
            entries,
            scratch,
        )
        .map(|step| step.scratch_bytes())
    };
    let past = |exceeded| Err(OrdinaryRootStepDenial::BoundExceeded(exceeded));

    // The result holds three routes: that is the first count past two.
    assert_eq!(check(3, u64::MAX).map(|_| ()), Ok(()));
    assert_eq!(check(2, u64::MAX), past(Exceeded::entries(3, 2)));
    assert_eq!(check(0, u64::MAX), past(Exceeded::entries(2, 0)));
    let need = check(3, u64::MAX).unwrap();
    assert_eq!(check(3, need), Ok(need));
    assert_eq!(check(3, need - 1), past(Exceeded::scratch(need, need - 1)));
    assert_eq!(
        VerifiedOrdinaryRootStep::maximum_recheck_heap_bytes(source, result, &projection, 3),
        Some(need)
    );
    assert_eq!(
        VerifiedOrdinaryRootStep::maximum_recheck_heap_bytes(source, result, &projection, 2),
        None
    );
    assert_eq!(
        OrdinaryRootStepDenial::BoundExceeded(Exceeded::entries(3, 2)).exceeded_bound(),
        Some(Exceeded::entries(3, 2))
    );
    assert_eq!(OrdinaryRootStepDenial::InvalidDelta.exceeded_bound(), None);
}

#[test]
fn the_free_ranges_a_step_would_build_are_held_to_the_entries() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let routes = [route(1, 1, 0)];
    let entries = [free_entry(8192, 12288, 1)];
    let (root, free) = snapshot(1, &routes, &entries, 3, None, format);
    let view = ReleasedInventoryView::new(&root, &free, &routes, &[], &entries);
    let past = |observed, limit| {
        Err(OrdinaryRootStepDenial::BoundExceeded(Exceeded::entries(
            observed, limit,
        )))
    };

    // Splitting the one free range leaves it one entry: more than none.
    let within = projection(route(3, 3, 8192));
    assert_eq!(delta::free_matches(view, view, &within, 0), past(1, 0));
    assert!(delta::free_matches(view, view, &within, 1).is_ok());

    // The source's next arena is 2. A placement in arena 3 opens two arenas,
    // each one more free range than the source's own.
    let two_arenas = projection(far_route(3));
    assert_eq!(delta::free_matches(view, view, &two_arenas, 1), past(2, 1));
    assert_eq!(delta::free_matches(view, view, &two_arenas, 2), past(3, 2));
    assert!(delta::free_matches(view, view, &two_arenas, 3).is_ok());
}
