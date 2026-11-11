//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryRuntimeFacadeFamily>(
            "workspaces/worth-query/crates/worth-query/src/runtime/support/facade_families.rs",
            "WorthQueryRuntimeFacadeFamily",
            128 * 1024,
        );
    }
}
