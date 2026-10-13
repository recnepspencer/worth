//! Binding retains the observed native handle without a bootstrap acquisition.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
#[test]
fn binding_a_truth_partition_keeps_handle_custody_and_its_listener() {
    let runtime =
        crate::relational_source::relational_test_support::runtime_with_declared_aspect_schema(
            worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
        );
    let source = RuntimeBridgeRelationalSource::for_shared_graph_role(
        Arc::new(Mutex::new(runtime)),
        "primary",
    )
    .unwrap();
    let acquisitions = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&acquisitions);
    let _guard = source.observe_runtime_locks_for_test(move || {
        observed.fetch_add(1, Ordering::SeqCst);
    });
    let bound = source
        .with_graph_partition(
            "primary",
            PartitionId::main(),
            worth_foundational::facade::TruthPartitionRole::new("primary").unwrap(),
        )
        .unwrap();
    assert_eq!(acquisitions.load(Ordering::SeqCst), 0);
    bound.runtime.with_runtime(|_| {});
    assert_eq!(acquisitions.load(Ordering::SeqCst), 1);
}
