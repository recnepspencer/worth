//! Exact issued lease retirement keeps its payload when the real index is busy.
use super::*;
use crate::tests::support::runtime_with_test_schema;

#[test]
fn result_snapshot_cleanup_basis_registry_contention_keeps_original_lease() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let registry = runtime
        .history
        .branch_cell(identity.branch_id())
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
    runtime.prepare_close().unwrap().finish();
}
