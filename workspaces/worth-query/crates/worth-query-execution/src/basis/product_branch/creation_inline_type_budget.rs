//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryProductBranchCreateError>("workspaces/worth-query/crates/worth-query-execution/src/basis/product_branch/creation.rs", "WorthQueryProductBranchCreateError", 128 * 1024);
    }
}
