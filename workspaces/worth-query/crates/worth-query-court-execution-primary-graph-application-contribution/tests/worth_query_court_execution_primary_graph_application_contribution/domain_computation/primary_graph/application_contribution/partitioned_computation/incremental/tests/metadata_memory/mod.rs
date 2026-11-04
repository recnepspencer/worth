//! Shared metadata keeps old grants; only changed paths need fresh room.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy;

#[test]
fn partition_metadata_refusal_preserves_prior_and_its_funded_view_lasts_until_final_drop() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = (*(*sealed.state_for_test()).typed_for_test())
        .downcast_ref::<RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let bound = (*prior.partitions_for_test()).bytes_for_test().unwrap();
    assert!(bound > 1);
    let count = (*prior.partitions_for_test()).len_for_test();
    let first_identity = *(*prior.partitions_for_test())
        .keys_for_test()
        .next()
        .unwrap();
    let original = (*(*prior.partitions_for_test())
        .get_for_test(&first_identity)
        .unwrap()
        .key_for_test())
    .0;
    let request = live_scope();
    let narrow = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, Some(1))),
        &request,
    );
    let mut candidate = (*prior.partitions_for_test()).fork_for_test();
    assert!(std::ptr::eq(
        (*prior.partitions_for_test())
            .get_for_test(&first_identity)
            .unwrap(),
        candidate.get_for_test(&first_identity).unwrap()
    ));
    let value = candidate.get_for_test(&first_identity).unwrap().clone();
    assert!(candidate
        .put_for_test(
            first_identity,
            value,
            narrow.allocation_policy_for_test().unwrap()
        )
        .is_err());
    assert_eq!(candidate.len_for_test(), count);
    assert!(std::ptr::eq(
        (*prior.partitions_for_test())
            .get_for_test(&first_identity)
            .unwrap(),
        candidate.get_for_test(&first_identity).unwrap()
    ));
    let capacity = bound * 4;
    let fresh = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(capacity),
        )),
        &request,
    );
    assert!(
        fresh.reserve(capacity).is_ok(),
        "sharing charged no unrelated backing"
    );
    let value = candidate.get_for_test(&first_identity).unwrap().clone();
    candidate
        .put_for_test(
            first_identity,
            value,
            fresh.allocation_policy_for_test().unwrap(),
        )
        .unwrap();
    let view = Arc::new(candidate);
    let clone = Arc::clone(&view);
    assert!(fresh.reserve(capacity).is_err());
    drop(view);
    assert!(
        fresh.reserve(capacity).is_err(),
        "a surviving view retains the changed path grant"
    );
    assert_eq!(
        (*clone.get_for_test(&first_identity).unwrap().key_for_test()).0,
        original
    );
    drop(clone);
    assert!(
        fresh.reserve(capacity).is_ok(),
        "final changed view releases its grant"
    );
    assert_eq!((*prior.partitions_for_test()).len_for_test(), count);
    assert_eq!(
        (*(*prior.partitions_for_test())
            .get_for_test(&first_identity)
            .unwrap()
            .key_for_test())
        .0,
        original
    );
}
