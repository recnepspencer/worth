use worth_runtime_bridge::facade::BridgeSubscriptionDeliveryWindowOpen;

fn cannot_reconstruct_rich_diagnostics(window: &BridgeSubscriptionDeliveryWindowOpen) {
    let _ = window.reconstruct_rich_diagnostics();
}

fn main() {}
