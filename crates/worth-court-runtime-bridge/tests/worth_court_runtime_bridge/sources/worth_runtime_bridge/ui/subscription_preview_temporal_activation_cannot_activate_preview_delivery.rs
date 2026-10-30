use worth_runtime_bridge::facade::{
    BridgePreviewTemporalSubscriptionActivationReady, BridgeSubscriptionConsumerContract,
    BridgeSubscriptionDeliveryCostProfile, BridgeSubscriptionPreviewBasisBinding, RuntimeBridge,
};

fn cannot_activate_preview_delivery_from_preview_temporal_ready(
    runtime: &RuntimeBridge,
    ready: BridgePreviewTemporalSubscriptionActivationReady,
    preview_basis: BridgeSubscriptionPreviewBasisBinding,
    cost_profile: BridgeSubscriptionDeliveryCostProfile,
    consumer_contract: BridgeSubscriptionConsumerContract,
) {
    let _ =
        runtime.activate_preview_subscription_delivery(ready, preview_basis, cost_profile, consumer_contract);
}

fn main() {}
