use worth_runtime_bridge::facade::{
    BridgeSubscriptionConsumerContract, BridgeSubscriptionDeliveryCostProfile,
    BridgeTemporalSubscriptionActivationReady, RuntimeBridge,
};

fn cannot_activate_ordinary_delivery_from_temporal_ready(
    runtime: &RuntimeBridge,
    ready: BridgeTemporalSubscriptionActivationReady,
    cost_profile: BridgeSubscriptionDeliveryCostProfile,
    consumer_contract: BridgeSubscriptionConsumerContract,
) {
    let _ = runtime.activate_subscription_delivery(ready, cost_profile, consumer_contract);
}

fn main() {}
