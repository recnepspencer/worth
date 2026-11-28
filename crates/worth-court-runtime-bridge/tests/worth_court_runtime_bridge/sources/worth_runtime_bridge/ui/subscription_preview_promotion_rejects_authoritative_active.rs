use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionPreviewLifecycleResidueEnvelope,
    BridgeSubscriptionPreviewWorkTrace, FinishedBridgePreviewSession, RuntimeBridge,
};

fn promote_authoritative_active(
    runtime: &RuntimeBridge,
    active: BridgeActiveSubscription,
    work_trace: &BridgeSubscriptionPreviewWorkTrace,
    promotion_record: &FinishedBridgePreviewSession,
    residue: &BridgeSubscriptionPreviewLifecycleResidueEnvelope,
) {
    let _ =
        runtime.admit_preview_lifecycle_promotion(&active, work_trace, residue, promotion_record);
}

fn main() {}
