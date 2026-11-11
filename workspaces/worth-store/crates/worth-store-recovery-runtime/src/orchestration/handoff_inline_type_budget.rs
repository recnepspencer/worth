//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PreparedHandoff>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/orchestration/handoff.rs", "PreparedHandoff", 128 * 1024);
        owner.field::<Result<
            construction::RejoinConstructionInput,
            worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial,
        >>("input");
        owner.field::<RecoveryHandoffBacking>("backing");
    }
    {
        let mut owner = census.owner::<ConstructedHandoff>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/orchestration/handoff.rs", "ConstructedHandoff", 128 * 1024);
        owner.field::<Result<(), worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial>>("result");
        owner.field::<RecoveryHandoffBacking>("backing");
    }
    super::backing::inline_type_budget::collect(census);
    super::completion::inline_type_budget::collect(census);
}
