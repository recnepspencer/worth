//! Retention refusal preserves existing custody before any replacement.
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::WorthQueryInvalidationResourceInstallation;

fn resources(maximum: u64) -> WorthQueryInvalidationResources {
    WorthQueryInvalidationResources::install(WorthQueryInvalidationResourceInstallation::bounded(
        1_000_000,
        16 * 1024 * 1024,
        maximum,
        16,
    ))
    .unwrap()
}

#[test]
fn a_tree_sized_allowance_refuses_a_forest_without_displacing_the_root() {
    let tree = retained_map_bytes::<usize, ()>(128).unwrap();
    let forest = retained_forest_bytes::<usize, ()>(128, 128).unwrap();
    let ticket = arc_bytes::<RetainedInvalidationCapacity>().unwrap();
    let root_bytes = state_bound(&MarkState::initial()).unwrap() + ticket;
    let maximum = root_bytes + tree + ticket;
    let resources = resources(maximum);
    let mut admission = InvalidationEditAdmission::new(resources.preflight_budget());
    let mut root = MarkState::initial();
    admit_first(&mut root, &resources, &mut admission).unwrap();
    let original = Arc::clone(root.retained_capacity.as_ref().unwrap());
    let before = resources.retained_capacity_bytes();

    assert!(matches!(
        reserve(&resources, forest, &mut admission),
        Err(CompanionPreflightStop::RetainedCompanionCapacityExhausted {
            requested, retained, maximum: actual_maximum,
        }) if requested == forest + ticket && retained == before && actual_maximum == maximum
    ));
    assert_eq!(resources.retained_capacity_bytes(), before);
    assert!(Arc::ptr_eq(
        root.retained_capacity.as_ref().unwrap(),
        &original
    ));
    assert!(root.settlements.is_empty());

    // The same allowance still admits one actual tree reservation. A refused
    // larger request did not consume its retained capacity or destroy custody.
    let allowed = reserve(&resources, tree, &mut admission).unwrap();
    assert_eq!(resources.retained_capacity_bytes(), maximum);
    drop(allowed);
    drop(original);
    drop(root);
    assert_eq!(resources.retained_capacity_bytes(), 0);
}

#[test]
fn an_overflowing_replacement_preserves_the_existing_reservation() {
    let resources = resources(1024 * 1024);
    let mut admission = InvalidationEditAdmission::new(resources.preflight_budget());
    let mut root = MarkState::initial();
    admit_first(&mut root, &resources, &mut admission).unwrap();
    let original = Arc::clone(root.retained_capacity.as_ref().unwrap());
    let before = resources.retained_capacity_bytes();
    assert!(matches!(
        admit_version(&mut root, u64::MAX, &resources, &mut admission),
        Err(CompanionPreflightStop::PreparationMemoryCounterOverflow)
    ));
    assert_eq!(resources.retained_capacity_bytes(), before);
    assert!(Arc::ptr_eq(
        root.retained_capacity.as_ref().unwrap(),
        &original
    ));
    assert!(root.settlements.is_empty());
    drop(original);
    drop(root);
    assert_eq!(resources.retained_capacity_bytes(), 0);
}
