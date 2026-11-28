use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionPreviewLifecycleResidueEnvelope, RuntimeBridge,
};

fn discard_authoritative_active(
    runtime: &RuntimeBridge,
    active: BridgeActiveSubscription,
    residue_index: BridgeSubscriptionPreviewLifecycleResidueEnvelope,
) {
    let _ = runtime.admit_preview_lifecycle_discard(active, residue_index);
}

fn main() {}
