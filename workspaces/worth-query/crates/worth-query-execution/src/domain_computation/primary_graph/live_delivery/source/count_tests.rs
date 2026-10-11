//! Count exhaustion refuses admission and cleanup remains safe while unwinding.
use super::*;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn subscription_count_exhaustion_refuses_without_panicking() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let product = runtime
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let source = WorthQueryLiveDeliverySource::default();
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
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
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
