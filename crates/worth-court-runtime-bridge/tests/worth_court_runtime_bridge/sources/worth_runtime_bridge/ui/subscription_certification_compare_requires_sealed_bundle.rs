use worth_runtime_bridge::facade::{
    BridgeSubscriptionCertificationBundleDraft, BridgeSubscriptionCertificationComparisonPlan,
    RuntimeBridge,
};

fn compare_draft_bundle(
    runtime: &RuntimeBridge,
    plan: BridgeSubscriptionCertificationComparisonPlan,
    draft: &BridgeSubscriptionCertificationBundleDraft,
) {
    let _report = runtime.compare_subscription_certification_bundles(plan, &draft, &draft);
}

fn main() {}
