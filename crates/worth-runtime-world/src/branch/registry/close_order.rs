//! Close visits the declared non-root branch inventory.
use crate::identity::ProductBranchIdentity;

pub(super) fn non_root_branches<'a>(
    branches: impl Iterator<Item = &'a ProductBranchIdentity>,
    root: Option<&ProductBranchIdentity>,
) -> Vec<ProductBranchIdentity> {
    let mut branches: Vec<_> = branches
        .filter(|branch| root != Some(*branch))
        .cloned()
        .collect();
    branches.sort();
    branches
}
