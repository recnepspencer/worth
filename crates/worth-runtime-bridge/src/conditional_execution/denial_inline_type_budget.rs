//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeConditionalDenialKind>(
            "crates/worth-runtime-bridge/src/conditional_execution/denial.rs",
            "BridgeConditionalDenialKind",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<BridgeConditionalDenial>(
            "crates/worth-runtime-bridge/src/conditional_execution/denial.rs",
            "BridgeConditionalDenial",
            128 * 1024,
        );
        owner.field::<BridgeConditionalDenialKind>("kind");
        owner.field::<String>("detail");
        owner
            .field::<super::super::BridgeInstalledConditionalLoweringCounters>("lowering_counters");
        owner
            .field::<super::super::BridgeConditionalExecutionCounters>("bridge_execution_counters");
        owner.field::<super::super::BridgeConditionalReentryCounters>("reentry_counters");
        owner.field::<worth_signal::facade::SignalConditionalDecisionCounters>("signal_counters");
        owner.field::<usize>("semantic_observation_reads");
    }
}
