use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionContinuationRejection, RuntimeBridge,
};

fn plan_from_rejection(
    runtime: &RuntimeBridge,
    active_subscription: &BridgeActiveSubscription,
    rejected_continuation: &BridgeSubscriptionContinuationRejection,
) {
    let _ = runtime.plan_subscription_continuation(active_subscription, rejected_continuation, 0);
}

fn main() {}
