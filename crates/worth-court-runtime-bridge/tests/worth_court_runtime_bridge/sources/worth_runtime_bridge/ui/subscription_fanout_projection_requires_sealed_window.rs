use worth_runtime_bridge::facade::{
    BridgeSubscriptionDeliveryWindowOpen, BridgeSubscriptionFanoutLayout, RuntimeBridge,
};

fn cannot_project_open_window(
    runtime: &RuntimeBridge,
    layout: &BridgeSubscriptionFanoutLayout,
    open: &BridgeSubscriptionDeliveryWindowOpen,
) {
    let _ = runtime.project_subscription_delivery_to_fanout(layout, open);
}

fn main() {}
