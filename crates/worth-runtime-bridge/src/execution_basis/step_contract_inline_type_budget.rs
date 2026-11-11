//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeManagedExecutionStepContract>(
            "crates/worth-runtime-bridge/src/execution_basis/step_contract.rs",
            "BridgeManagedExecutionStepContract",
            128 * 1024,
        );
        owner.field::<BridgeManagedExecutionStepContractIdentity>("identity");
        owner.field::<Arc<str>>("safe_point_family");
        owner.field::<u64>("queue_depth_ceiling");
        owner.field::<Option<BridgeManagedExecutionStepLimits>>("step_limits");
        owner.field::<Option<u64>>("deadline_nanos");
        owner.field::<bool>("partial_effects_may_remain");
    }
}
