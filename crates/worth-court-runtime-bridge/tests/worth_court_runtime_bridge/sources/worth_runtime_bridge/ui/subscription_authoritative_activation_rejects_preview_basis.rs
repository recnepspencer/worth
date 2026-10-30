use worth_runtime_bridge::facade::{
    BridgeSubscriptionActivationReady, BridgeSubscriptionConsumerContract,
    BridgeSubscriptionPreviewBasisBinding, RuntimeBridge,
};

fn activate_with_preview_basis(
    runtime: &RuntimeBridge,
    ready: BridgeSubscriptionActivationReady,
    preview_basis: BridgeSubscriptionPreviewBasisBinding,
    consumer_contract: BridgeSubscriptionConsumerContract,
) {
    let _ = runtime.activate_subscription_delivery(ready, preview_basis, consumer_contract);
}

fn main() {}
