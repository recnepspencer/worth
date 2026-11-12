//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<PhysicalMutationProgressPhase>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/mutation/outcome.rs", "PhysicalMutationProgressPhase", 128 * 1024);
    }
    {
        let _owner = census.owner::<PhysicalMutationTerminalFact>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/mutation/outcome.rs", "PhysicalMutationTerminalFact", 128 * 1024);
    }
}
