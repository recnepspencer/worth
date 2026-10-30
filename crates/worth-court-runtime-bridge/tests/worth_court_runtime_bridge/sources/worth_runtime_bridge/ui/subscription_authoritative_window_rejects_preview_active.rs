use worth_runtime_bridge::facade::{
    BridgePreviewActiveSubscription, BridgeSubscriptionDeliveryFamilyKind, RuntimeBridge,
};

fn open_authoritative_window_from_preview(
    runtime: &RuntimeBridge,
    preview_active: &BridgePreviewActiveSubscription,
) {
    let _ = runtime.open_subscription_delivery_window(
        preview_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
    );
}

fn main() {}
