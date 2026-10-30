use worth_runtime_bridge::facade::{BridgeActiveSubscription, RuntimeBridge};

fn admit_unsealed_stream_checkpoint(
    runtime: &RuntimeBridge,
    active_subscription: &BridgeActiveSubscription,
    unsealed_stream_checkpoint: &str,
) {
    let _ = runtime.admit_subscription_resume(active_subscription, unsealed_stream_checkpoint);
}

fn main() {}
