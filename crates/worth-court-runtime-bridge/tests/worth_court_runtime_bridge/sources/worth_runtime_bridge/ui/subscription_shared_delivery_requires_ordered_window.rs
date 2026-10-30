use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeMixedCauseOrdering, BridgeSubscriptionFanoutLayout, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let active: BridgeActiveSubscription = fake();
    let ordering: BridgeMixedCauseOrdering = fake();
    let fanout_layout: BridgeSubscriptionFanoutLayout = fake();
    let _ = runtime.plan_shared_subscription_delivery(&active, &ordering, &fanout_layout);
}
