//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryProductBranch>("workspaces/worth-query/crates/worth-query-execution/src/basis/product_branch/selection.rs", "WorthQueryProductBranch", 128 * 1024);
        owner.field::<worth_runtime_world::facade::ProductBranchIncarnation>("occurrence");
    }
}
