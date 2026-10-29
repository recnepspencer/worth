use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionAcknowledgementFrontier,
    BridgeSubscriptionDuplicateReplayPolicyKind, RuntimeBridge,
};

fn publish_unsealed_acknowledgement_frontier(
    runtime: &RuntimeBridge,
    frontier: BridgeSubscriptionAcknowledgementFrontier,
    active: &BridgeActiveSubscription,
) {
    let _ = runtime.publish_subscription_checkpoint(
        frontier,
        active,
        BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
}

fn main() {}
