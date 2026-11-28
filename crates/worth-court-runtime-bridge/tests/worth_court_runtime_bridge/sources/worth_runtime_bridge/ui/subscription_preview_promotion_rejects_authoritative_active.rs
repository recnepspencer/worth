use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgePreviewPromotionRecord,
    BridgeSubscriptionPreviewLifecycleResidueEnvelope, BridgeSubscriptionPreviewWorkTrace,
    RuntimeBridge,
};

fn promote_authoritative_active(
    runtime: &RuntimeBridge,
    active: BridgeActiveSubscription,
    work_trace: &BridgeSubscriptionPreviewWorkTrace,
    promotion_record: &BridgePreviewPromotionRecord,
    residue: &BridgeSubscriptionPreviewLifecycleResidueEnvelope,
) {
    let _ =
        runtime.admit_preview_lifecycle_promotion(&active, work_trace, residue, promotion_record);
}

fn main() {}
