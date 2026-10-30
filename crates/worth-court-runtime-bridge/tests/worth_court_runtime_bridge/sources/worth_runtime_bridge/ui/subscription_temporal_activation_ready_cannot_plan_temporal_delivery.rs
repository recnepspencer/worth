use worth_runtime_bridge::facade::{
    BridgeSubscriptionDeliveryFamilyKind, BridgeTemporalSubscriptionActivationReady,
    BridgeTemporalDeliveryWindowPlan,
};

fn fake<T>() -> T {
    panic!("type-only")
}

fn main() {
    let ready: BridgeTemporalSubscriptionActivationReady = fake();
    let _ = BridgeTemporalDeliveryWindowPlan::plan(
        &ready,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
}
