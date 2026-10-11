//! Close visits the declared non-root branch inventory.
use super::installed_entries::InstalledProductBranches;
use crate::identity::ProductBranchIdentity;

pub(super) fn non_root_branches(
    branches: &InstalledProductBranches,
    root: Option<&ProductBranchIdentity>,
) -> Vec<ProductBranchIdentity> {
    // The installed inventory carries declared branch order in its BTreeSet.
    branches
        .keys()
        .filter(|branch| root != Some(*branch))
        .cloned()
        .collect()
}
