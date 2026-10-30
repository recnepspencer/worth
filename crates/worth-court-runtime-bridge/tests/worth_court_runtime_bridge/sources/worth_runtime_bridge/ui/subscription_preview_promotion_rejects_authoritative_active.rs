use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgePreviewPromotionRecord, BridgeSubscriptionActivationReady,
    BridgeSubscriptionPreviewWorkTrace, RuntimeBridge,
};

fn promote_authoritative_active(
    runtime: &RuntimeBridge,
    active: BridgeActiveSubscription,
    work_trace: &BridgeSubscriptionPreviewWorkTrace,
    promotion_record: &BridgePreviewPromotionRecord,
    promoted_ready: &BridgeSubscriptionActivationReady,
) {
    let _ = runtime.record_preview_authoritative_boundary(
        active,
        work_trace,
        promotion_record,
        promoted_ready,
    );
}

fn main() {}
