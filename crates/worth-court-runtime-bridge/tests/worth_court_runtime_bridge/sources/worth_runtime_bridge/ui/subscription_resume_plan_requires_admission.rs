use worth_runtime_bridge::facade::{
    BridgeSubscriptionCheckpoint, BridgeSubscriptionDuplicateReplayPolicyKind, RuntimeBridge,
};

fn plan_from_checkpoint(runtime: &RuntimeBridge, checkpoint: BridgeSubscriptionCheckpoint) {
    let _ = BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers;
    let _ = runtime.plan_subscription_resume(checkpoint);
}

fn main() {}
