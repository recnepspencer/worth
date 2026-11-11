//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<PhysicalMutationCheckpoint>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/lifecycle/yieldpoint.rs", "PhysicalMutationCheckpoint", 128 * 1024);
    }
}
