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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        branch::ProductBranchName, lifecycle::owner::RuntimeWorldOwnerConstructionContract,
    };

    #[test]
    fn equal_contents_close_in_the_same_declared_branch_sequence() {
        let construction = RuntimeWorldOwnerConstructionContract::new().unwrap();
        let branch = |name| {
            ProductBranchIdentity::issued(
                construction.owner_identity(),
                ProductBranchName::try_new(name).unwrap(),
            )
        };
        let alpha = branch("alpha");
        let zulu = branch("zulu");
        let root = branch("root");
        let selected = non_root_branches([&zulu, &root, &alpha].into_iter(), Some(&root));
        assert_eq!(
            selected
                .iter()
                .map(|branch| branch.name().as_str())
                .collect::<Vec<_>>(),
            ["alpha", "zulu"]
        );
    }
}
