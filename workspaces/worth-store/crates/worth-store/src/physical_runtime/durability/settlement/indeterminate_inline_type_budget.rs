//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<PhysicalMutationIndeterminateStage>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/settlement/indeterminate.rs", "PhysicalMutationIndeterminateStage", 128 * 1024);
    }
    super::root_preparation::inline_type_budget::collect(census);
}
