//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalStoreWorkRuntime>("workspaces/worth-store/crates/worth-store/src/physical_runtime/instance/work_runtime.rs", "PhysicalStoreWorkRuntime", 128 * 1024);
        owner.field::<PhysicalWorkSubmissionOwner>("submission");
        owner.field::<PhysicalWorkSignalOwner>("signal");
        owner.field::<PhysicalWorkExecutor>("executor");
        owner.field::<ServingHealth>("health");
        owner.field::<crate::physical_runtime::work::PhysicalEffectRecoveryInventory>("recovery");
        owner.field::<Arc<PhysicalExecutionGate>>("gate");
    }
}
