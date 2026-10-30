use worth_runtime_bridge::facade::{
    BridgeMixedCauseDeliveryWindowPlan, BridgeMixedCauseOrderingRequest,
    BridgeMixedCauseOrderingLaneKind, BridgeSubscriptionDeliveryFamilyKind,
};

fn main() {
    let request = BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        Vec::new(),
    );
    let _ = BridgeMixedCauseDeliveryWindowPlan::plan(
        &request,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
}
