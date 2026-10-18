//! Shared metadata keeps old grants; only changed paths need fresh room.
use super::super::retained::RetainedPartitions;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;

#[test]
fn partition_metadata_refusal_preserves_prior_and_its_funded_view_lasts_until_final_drop() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = sealed
        .state
        .typed
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let bound = prior.partitions.bytes().unwrap();
    assert!(bound > 1);
    let count = prior.partitions.len();
    let first_identity = *prior.partitions.keys().next().unwrap();
    let original = prior.partitions.get(&first_identity).unwrap().key.0;
    let request = live_scope();
    let narrow = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, 1)),
        &request,
    );
    let mut candidate = prior.partitions.fork();
    assert!(std::ptr::eq(
        prior.partitions.get(&first_identity).unwrap(),
        candidate.get(&first_identity).unwrap()
    ));
    let value = candidate.get(&first_identity).unwrap().clone();
    assert!(candidate
        .put(first_identity, value, narrow.allocation_policy().unwrap())
        .is_err());
    assert_eq!(candidate.len(), count);
    assert!(std::ptr::eq(
        prior.partitions.get(&first_identity).unwrap(),
        candidate.get(&first_identity).unwrap()
    ));
    let capacity = bound * 4;
    let fresh = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, capacity)),
        &request,
    );
    assert!(
        fresh.reserve(capacity).is_ok(),
        "sharing charged no unrelated backing"
    );
    let value = candidate.get(&first_identity).unwrap().clone();
    candidate
        .put(first_identity, value, fresh.allocation_policy().unwrap())
        .unwrap();
    let view = Arc::new(candidate);
    let clone = Arc::clone(&view);
    assert!(fresh.reserve(capacity).is_err());
    drop(view);
    assert!(
        fresh.reserve(capacity).is_err(),
        "a surviving view retains the changed path grant"
    );
    assert_eq!(clone.get(&first_identity).unwrap().key.0, original);
    drop(clone);
    assert!(
        fresh.reserve(capacity).is_ok(),
        "final changed view releases its grant"
    );
    assert_eq!(prior.partitions.len(), count);
    assert_eq!(
        prior.partitions.get(&first_identity).unwrap().key.0,
        original
    );
}
