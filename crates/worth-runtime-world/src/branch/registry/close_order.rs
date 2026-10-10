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
    use std::collections::HashMap;

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
        for _ in 0..16 {
            let left = HashMap::from([(zulu.clone(), ()), (root.clone(), ()), (alpha.clone(), ())]);
            let right =
                HashMap::from([(alpha.clone(), ()), (root.clone(), ()), (zulu.clone(), ())]);
            // The production close path consumes precisely this selection;
            // values grant no head authority and do not affect branch order.
            let a = non_root_branches(left.keys(), Some(&root));
            let b = non_root_branches(right.keys(), Some(&root));
            assert_eq!(a, b);
            assert_eq!(
                a.iter()
                    .map(|branch| branch.name().as_str())
                    .collect::<Vec<_>>(),
                ["alpha", "zulu"]
            );
        }
    }
}
