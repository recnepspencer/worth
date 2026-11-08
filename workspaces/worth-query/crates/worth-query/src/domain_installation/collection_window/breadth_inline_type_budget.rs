//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryCollectionWindowBreadth>("workspaces/worth-query/crates/worth-query/src/domain_installation/collection_window/breadth.rs", "WorthQueryCollectionWindowBreadth", 128 * 1024);
        owner.field::<u32>("viewport_rows");
        owner.field::<u32>("overscan_before");
        owner.field::<u32>("overscan_after");
        owner.field::<u32>("mounting_budget");
        owner.field::<u32>("admitted_width");
    }
}
