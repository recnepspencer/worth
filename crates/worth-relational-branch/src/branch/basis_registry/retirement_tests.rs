//! Exact issued lease retirement keeps its payload when the real index is busy.
use super::*;
use crate::branch::basis_admission_tests::observed_empty_branch;

#[test]
fn result_snapshot_cleanup_basis_registry_contention_keeps_original_lease() {
    let (history, basis) = observed_empty_branch(1);
    let registry = history
        .branch_cell(&history.main_branch)
        .unwrap()
        .basis_registry
        .clone();
    let mut inner =
        Arc::into_inner(basis.inner).expect("this actual observation has no independent reader");
    let mut lease = inner.registry_lease.take().unwrap();
    let locked = registry.retained.lock().unwrap();
    let original = locked.entries.get(&lease.key).unwrap().registration_id;
    assert!(!lease.try_retire());
    assert!(!lease.retired);
    assert_eq!(
        locked.entries.get(&lease.key).unwrap().registration_id,
        original
    );
    drop(locked);
    assert!(lease.try_retire());
    assert!(lease.retired);
    assert!(registry.retained.lock().unwrap().entries.is_empty());
    let metrics = registry.metrics.snapshot();
    assert!(lease.try_retire());
    assert_eq!(registry.metrics.snapshot(), metrics);
    drop(inner);
}
