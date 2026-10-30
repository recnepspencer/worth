use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionPreviewResidueScopeIndex, RuntimeBridge,
};

fn discard_authoritative_active(
    runtime: &RuntimeBridge,
    active: BridgeActiveSubscription,
    residue_index: BridgeSubscriptionPreviewResidueScopeIndex,
) {
    let _ = runtime.prove_preview_scope_discard_residue(active, residue_index);
}

fn main() {}
