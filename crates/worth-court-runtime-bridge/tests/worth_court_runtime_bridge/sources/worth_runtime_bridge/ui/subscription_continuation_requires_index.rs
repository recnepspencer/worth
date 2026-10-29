use worth_runtime_bridge::facade::{
    BridgeActiveSubscription, BridgeSubscriptionContinuationCandidateInput, RuntimeBridge,
};

fn plan_from_unindexed_continuation_candidate(
    runtime: &RuntimeBridge,
    active: &BridgeActiveSubscription,
    candidate: BridgeSubscriptionContinuationCandidateInput,
) {
    let _ = runtime.plan_subscription_continuation(active, &candidate, 0);
}

fn main() {}
