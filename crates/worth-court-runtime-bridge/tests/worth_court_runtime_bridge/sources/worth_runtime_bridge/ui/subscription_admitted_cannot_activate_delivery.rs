use worth_runtime_bridge::facade::{
    AdmittedBridgeSubscription, BridgeSubscriptionConsumerContract,
    BridgeSubscriptionDeliveryCostProfile, RuntimeBridge,
};

fn cannot_activate_delivery_from_unactivated_subscription_admission(
    runtime: &RuntimeBridge,
    admitted: AdmittedBridgeSubscription,
    cost_profile: BridgeSubscriptionDeliveryCostProfile,
    consumer: BridgeSubscriptionConsumerContract,
) {
    let _ = runtime.activate_subscription_delivery(admitted, cost_profile, consumer);
}

fn main() {}
