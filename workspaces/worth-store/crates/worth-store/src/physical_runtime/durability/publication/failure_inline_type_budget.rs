//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<PhysicalRootPublicationPreparationOutcome>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/publication/failure.rs", "PhysicalRootPublicationPreparationOutcome", 128 * 1024);
    }
}
