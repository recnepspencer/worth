//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeExecutionBasisDenialKind>(
            "crates/worth-runtime-bridge/src/execution_basis/denial.rs",
            "BridgeExecutionBasisDenialKind",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<BridgeExecutionBasisDenial>(
            "crates/worth-runtime-bridge/src/execution_basis/denial.rs",
            "BridgeExecutionBasisDenial",
            128 * 1024,
        );
        owner.field::<BridgeExecutionBasisDenialKind>("kind");
        owner.field::<String>("detail");
        owner.field::<BridgeExecutionBasisCounters>("counters");
    }
}
