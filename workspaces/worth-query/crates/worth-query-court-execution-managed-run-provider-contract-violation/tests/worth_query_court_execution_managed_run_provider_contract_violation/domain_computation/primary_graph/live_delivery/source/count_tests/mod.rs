//! Count exhaustion refuses admission and cleanup remains safe while unwinding.
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::live_delivery::source::subscription::WorthQueryLiveSubscriptionCapacityExhausted;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::live_delivery::source::WorthQueryLiveDeliverySource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::live_delivery::source::WorthQueryLiveProductKey;

#[test]
fn subscription_count_exhaustion_refuses_without_panicking() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let product = runtime
        .admit_product_branch_for_test(runtime.default_branch_for_test())
        .unwrap();
    let source = WorthQueryLiveDeliverySource::default();
    // Exercise the real partition and genuine World observation, without issuing
    // a Native interpretation or claiming this proves Source routing.
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.execution_request_for_for_test(runtime).unwrap();
            let pinned = source
                .open_predicate_for_test(&execution, product.observation())
                .unwrap();
            let first = source
                .begin_predicate_preparation_for_test(&execution, product.observation())
                .unwrap();
            let second = source
                .begin_predicate_preparation_for_test(&execution, product.observation())
                .unwrap();
            assert!(source
                .join_predicate_opening_for_test(&execution, &pinned, product.observation())
                .is_err());
            drop(first);
            assert!(source
                .join_predicate_opening_for_test(&execution, &pinned, product.observation())
                .is_err());
            drop(second);
            let opening = source
                .join_predicate_opening_for_test(&execution, &pinned, product.observation())
                .unwrap();
            assert!(source
                .begin_predicate_preparation_for_test(&execution, product.observation())
                .is_err());
            assert!(source.reserve_for_test(&execution, product.observation()).is_err());
            drop(opening);
            let preparation = source
                .begin_predicate_preparation_for_test(&execution, product.observation())
                .unwrap();
            drop(pinned);
            assert_eq!(
                source.active_partition_count_for_test(),
                1,
                "preparation pins the actual partition after the last subscriber"
            );
            drop(preparation);
            assert_eq!(source.active_partition_count_for_test(), 0);
            let pinned = source
                .open_predicate_for_test(&execution, product.observation())
                .unwrap();
            source.invalidate_predicates(product.observation());
            assert!(source
                .join_predicate_opening_for_test(&execution, &pinned, product.observation())
                .is_err());
            assert!(source
                .validate_predicate_progress_for_test(&execution, &pinned, product.observation())
                .is_err());
            drop(pinned);
            assert_eq!(source.active_partition_count_for_test(), 0);
        })
        .unwrap();
    (*source.state_for_test()).lock().unwrap().subscriber_count = usize::MAX;
    let attempt = catch_unwind(AssertUnwindSafe(|| source.open_for_test(product.observation())));
    assert!(
        attempt.is_ok(),
        "exhaustion must return a refusal, not panic"
    );
    assert!(matches!(
        attempt.unwrap(),
        Err(WorthQueryLiveSubscriptionCapacityExhausted)
    ));
    let state = (*source.state_for_test()).lock().unwrap();
    assert_eq!(state.subscriber_count, usize::MAX);
    assert!(state.partitions.is_empty());
}

#[test]
fn subscription_drop_saturates_underflow_without_panicking() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let product = runtime
        .admit_product_branch_for_test(runtime.default_branch_for_test())
        .unwrap();
    let source = WorthQueryLiveDeliverySource::default();
    let subscription = source.open_for_test(product.observation()).unwrap();
    {
        let mut state = (*source.state_for_test()).lock().unwrap();
        state.subscriber_count = 0;
        let key = WorthQueryLiveProductKey::from_observation_for_test(product.observation());
        state.partitions.get_mut(&key).unwrap().subscriber_count = 0;
    }
    assert!(
        catch_unwind(AssertUnwindSafe(|| drop(subscription))).is_ok(),
        "subscription cleanup cannot panic"
    );
    let state = (*source.state_for_test()).lock().unwrap();
    assert_eq!(state.subscriber_count, 0);
    assert!(state.partitions.is_empty());
}
