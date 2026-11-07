//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<PhysicalRootNamespaceDurabilityOutcome>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/namespace_durability.rs", "PhysicalRootNamespaceDurabilityOutcome", 128 * 1024);
    }
}
