//! Count exhaustion refuses admission and cleanup remains safe while unwinding.
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::live_delivery::source::subscription;
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
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let source = WorthQueryLiveDeliverySource::default();
    // Exercise the real partition and genuine World observation, without issuing
    // a Native interpretation or claiming this proves Source routing.
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase.execution_request_for(runtime).unwrap();
            let pinned = source
                .open_predicate(&execution, product.observation())
                .unwrap();
            let first = source
                .begin_predicate_preparation(&execution, product.observation())
                .unwrap();
            let second = source
                .begin_predicate_preparation(&execution, product.observation())
                .unwrap();
            assert!(source
                .join_predicate_opening(&execution, &pinned, product.observation())
                .is_err());
            drop(first);
            assert!(source
                .join_predicate_opening(&execution, &pinned, product.observation())
                .is_err());
            drop(second);
            let opening = source
                .join_predicate_opening(&execution, &pinned, product.observation())
                .unwrap();
            assert!(source
                .begin_predicate_preparation(&execution, product.observation())
                .is_err());
            assert!(source.reserve(&execution, product.observation()).is_err());
            drop(opening);
            let preparation = source
                .begin_predicate_preparation(&execution, product.observation())
                .unwrap();
            drop(pinned);
            assert_eq!(
                source.active_partition_count(),
                1,
                "preparation pins the actual partition after the last subscriber"
            );
            drop(preparation);
            assert_eq!(source.active_partition_count(), 0);
            let pinned = source
                .open_predicate(&execution, product.observation())
                .unwrap();
            source.invalidate_predicates(product.observation());
            assert!(source
                .join_predicate_opening(&execution, &pinned, product.observation())
                .is_err());
            assert!(source
                .validate_predicate_progress(&execution, &pinned, product.observation())
                .is_err());
            drop(pinned);
            assert_eq!(source.active_partition_count(), 0);
        })
        .unwrap();
    source.state.lock().unwrap().subscriber_count = usize::MAX;
    let attempt = catch_unwind(AssertUnwindSafe(|| source.open(product.observation())));
    assert!(
        attempt.is_ok(),
        "exhaustion must return a refusal, not panic"
    );
    assert!(matches!(
        attempt.unwrap(),
        Err(WorthQueryLiveSubscriptionCapacityExhausted)
    ));
    let state = source.state.lock().unwrap();
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
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let source = WorthQueryLiveDeliverySource::default();
    let subscription = source.open(product.observation()).unwrap();
    {
        let mut state = source.state.lock().unwrap();
        state.subscriber_count = 0;
        let key = WorthQueryLiveProductKey::from_observation(product.observation());
        state.partitions.get_mut(&key).unwrap().subscriber_count = 0;
    }
    assert!(
        catch_unwind(AssertUnwindSafe(|| drop(subscription))).is_ok(),
        "subscription cleanup cannot panic"
    );
    let state = source.state.lock().unwrap();
    assert_eq!(state.subscriber_count, 0);
    assert!(state.partitions.is_empty());
}
