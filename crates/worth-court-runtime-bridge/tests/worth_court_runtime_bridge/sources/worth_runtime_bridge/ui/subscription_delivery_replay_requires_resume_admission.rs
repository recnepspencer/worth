use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionCheckpoint,
    BridgeSubscriptionRetainedDeliveryWindowSeed, RuntimeBridge,
};

fn replay_from_checkpoint(
    runtime: &RuntimeBridge,
    active: &BridgeActiveSubscription,
    checkpoint: BridgeSubscriptionCheckpoint,
    seeds: Vec<BridgeSubscriptionRetainedDeliveryWindowSeed>,
) {
    let _ = runtime.plan_subscription_delivery_replay(active, checkpoint, seeds);
}

fn main() {}
