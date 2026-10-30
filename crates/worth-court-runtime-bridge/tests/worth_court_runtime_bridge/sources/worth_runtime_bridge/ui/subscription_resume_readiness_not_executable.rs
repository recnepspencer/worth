use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionDeliveryReplayPlan,
    BridgeSubscriptionReplayReadiness, RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let runtime = fake::<RuntimeBridge>();
    let active = fake::<BridgeActiveSubscription>();
    let readiness = fake::<BridgeSubscriptionReplayReadiness>();
    let _plan: Result<BridgeSubscriptionDeliveryReplayPlan, _> =
        runtime.plan_subscription_delivery_replay(&active, readiness, vec![]);
}
