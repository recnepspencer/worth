//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeExactOutputComparison>(
            "crates/worth-runtime-bridge/src/conditional_execution/output_comparison.rs",
            "BridgeExactOutputComparison",
            128 * 1024,
        );
        owner.field::<BridgeConditionalCausalObservation>("baseline");
        owner.field::<BridgeConditionalCausalObservation>("candidate");
    }
    {
        let _owner = census.owner::<BridgeOutputComparisonStage>(
            "crates/worth-runtime-bridge/src/conditional_execution/output_comparison.rs",
            "BridgeOutputComparisonStage",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<BridgeOutputComparisonDenial>(
            "crates/worth-runtime-bridge/src/conditional_execution/output_comparison.rs",
            "BridgeOutputComparisonDenial",
            128 * 1024,
        );
        owner.field::<BridgeOutputComparisonStage>("stage");
        owner.field::<BridgeConditionalDenial>("cause");
    }
}
