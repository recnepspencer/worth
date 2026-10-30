use worth_runtime_bridge::facade::{
    BridgeSubscriptionActivationReady, BridgeSubscriptionDeliveryFamilyKind, RuntimeBridge,
};

fn cannot_open_from_activation_ready(
    runtime: &RuntimeBridge,
    ready: &BridgeSubscriptionActivationReady,
) {
    let _ = runtime.open_subscription_delivery_window(
        ready,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
    );
}

fn main() {}
