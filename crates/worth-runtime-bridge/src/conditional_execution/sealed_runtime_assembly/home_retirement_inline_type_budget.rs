//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<FundedBridgeAssemblyRetirement>("crates/worth-runtime-bridge/src/conditional_execution/sealed_runtime_assembly/home_retirement.rs", "FundedBridgeAssemblyRetirement", 128 * 1024);
        owner.field::<std::sync::Mutex<Box<BridgeAssemblyRetirement>>>("state");
        owner.field::<Option<ExecutionMemoryReservation>>("_funding");
    }
    {
        let mut owner = census.owner::<BridgeAssemblyRetirement>("crates/worth-runtime-bridge/src/conditional_execution/sealed_runtime_assembly/home_retirement.rs", "BridgeAssemblyRetirement", 128 * 1024);
        owner
            .field::<Option<PreparedInitialEmptySignalStop<(), (), (), (), ()>>>("prepared_signal");
        owner.field::<Option<StoppedInitialEmptySignalAllocation<(), (), (), (), ()>>>(
            "_stopped_signal",
        );
        owner
            .field::<Option<crate::home_retirement::RetiringWorldCorrespondence>>("correspondence");
        owner.field::<Option<RetiringRuntimeBridge>>("bridge");
        owner.field::<Option<RetiredRuntimeBridge>>("_retired_bridge");
        owner.field::<AssemblyRetirementStage>("stage");
    }
    {
        let _owner = census.owner::<AssemblyRetirementStage>("crates/worth-runtime-bridge/src/conditional_execution/sealed_runtime_assembly/home_retirement.rs", "AssemblyRetirementStage", 128 * 1024);
    }
}
