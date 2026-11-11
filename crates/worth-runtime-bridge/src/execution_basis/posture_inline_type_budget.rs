//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<BridgeExecutionBasisPosture>(
            "crates/worth-runtime-bridge/src/execution_basis/posture.rs",
            "BridgeExecutionBasisPosture",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<BridgeManagedExecutionBasis>(
            "crates/worth-runtime-bridge/src/execution_basis/posture.rs",
            "BridgeManagedExecutionBasis",
            128 * 1024,
        );
        owner.field::<BridgeManagedExecutionStepContract>("step_contract");
        owner.field::<ResourceManagedQueueBinding>("queue");
        owner.field::<u64>("occupancy_width");
    }
}
