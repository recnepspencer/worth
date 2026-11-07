//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryProductBranchReadIdentity>("workspaces/worth-query/crates/worth-query-execution/src/basis/product_branch/read_identity.rs", "WorthQueryProductBranchReadIdentity", 128 * 1024);
        owner.field::<super::super::read_position::WorthQueryProductBranchReadPosition>("position");
        owner.field::<CompositeBasisKey>("basis");
    }
}
