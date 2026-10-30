use worth_runtime_bridge::facade::{BridgeSubscriptionCertificationBundleDraft, RuntimeBridge};

fn main() {
    fn build_index(runtime: &RuntimeBridge, draft: &BridgeSubscriptionCertificationBundleDraft) {
        let _ = runtime.build_subscription_offline_audit_bundle_index(vec![draft]);
    }
}
