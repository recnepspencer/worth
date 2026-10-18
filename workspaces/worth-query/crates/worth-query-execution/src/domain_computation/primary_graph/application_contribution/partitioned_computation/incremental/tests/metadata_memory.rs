//! Real retained metadata admits a fresh view before sharing any root.
use super::super::retained::RetainedPartitions;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial;

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
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, bound - 1)),
        &request,
    );
    assert!(matches!(
        prior.partitions.fork(&narrow),
        Err(WorthQueryManagedComputationResourceDenial::MemoryLimit { .. })
    ));
    assert_eq!(prior.partitions.len(), count);
    assert_eq!(
        prior.partitions.get(&first_identity).unwrap().key.0,
        original
    );
    let fresh = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, bound)),
        &request,
    );
    let view = Arc::new(prior.partitions.fork(&fresh).unwrap());
    assert!(Arc::ptr_eq(
        prior.partitions.get(&first_identity).unwrap(),
        view.get(&first_identity).unwrap()
    ));
    let clone = Arc::clone(&view);
    assert!(fresh.reserve(bound).is_err());
    drop(view);
    assert!(
        fresh.reserve(bound).is_err(),
        "the surviving view still owns the new backing grant"
    );
    assert_eq!(clone.get(&first_identity).unwrap().key.0, original);
    drop(clone);
    assert!(
        fresh.reserve(bound).is_ok(),
        "the final view releases its own grant without affecting the prior"
    );
    assert_eq!(prior.partitions.len(), count);
}
