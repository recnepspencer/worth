//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeConditionalDecisionEvidence>(
            "crates/worth-runtime-bridge/src/conditional_execution/decision_evidence.rs",
            "BridgeConditionalDecisionEvidence",
            128 * 1024,
        );
        owner.field::<std::sync::Arc<BridgeRetainedConditionalDecisionCore>>("core");
        owner.field::<std::sync::Arc<str>>("query_binding_identity");
        owner.field::<u64>("query_capability_identity");
        owner.field::<BridgeConditionalReentryCounters>("reentry_counters");
        owner.field::<Option<SignalInvalidationExecutionReceipt>>("performed_signal_invalidation");
        owner.field::<super::super::retention::BridgeRetentionReservation>("_reservation");
    }
}
