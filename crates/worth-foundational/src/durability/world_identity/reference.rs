use super::{
    DurableCompositeCommitIdentity, DurableProductBranchIdentity, DurableProductBranchIncarnation,
    DurableProductReferenceGeneration,
};

/// An immutable, runtime-neutral World product head. Recorded data grants
/// no live branch admission, publication or issuance authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableProductBranchReference {
    branch: DurableProductBranchIdentity,
    incarnation: DurableProductBranchIncarnation,
    generation: DurableProductReferenceGeneration,
    commit: DurableCompositeCommitIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DurableProductReferenceOwnerMismatch;

impl DurableProductBranchReference {
    pub fn from_recorded(
        branch: DurableProductBranchIdentity,
        incarnation: DurableProductBranchIncarnation,
        generation: DurableProductReferenceGeneration,
        commit: DurableCompositeCommitIdentity,
    ) -> Result<Self, DurableProductReferenceOwnerMismatch> {
        if branch.owner() != incarnation.owner() || branch.owner() != commit.owner() {
            return Err(DurableProductReferenceOwnerMismatch);
        }
        Ok(Self {
            branch,
            incarnation,
            generation,
            commit,
        })
    }

    pub fn branch(&self) -> &DurableProductBranchIdentity {
        &self.branch
    }
    pub const fn incarnation(&self) -> DurableProductBranchIncarnation {
        self.incarnation
    }
    pub const fn generation(&self) -> DurableProductReferenceGeneration {
        self.generation
    }
    pub const fn selected_commit(&self) -> DurableCompositeCommitIdentity {
        self.commit
    }
}

impl super::super::identity_seal::Sealed for DurableProductBranchReference {}
impl super::super::DurableIdentity for DurableProductBranchReference {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::{DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity};

    #[test]
    fn recorded_product_heads_refuse_mixed_home_owners() {
        let owner = |namespace| {
            DurableWorldOwnerIdentity::from_recorded(
                HomeIdentity::from_observed_bytes([namespace; 16]).unwrap(),
                WorldOwnerSlotIdentity::from_recorded(1),
            )
        };
        let first = owner(1);
        let foreign = owner(2);
        let branch = DurableProductBranchIdentity::from_recorded(first, "main".to_owned()).unwrap();
        let generation = DurableProductReferenceGeneration::from_recorded(7);
        for (incarnation_owner, commit_owner) in [(foreign, first), (first, foreign)] {
            assert_eq!(
                DurableProductBranchReference::from_recorded(
                    branch.clone(),
                    DurableProductBranchIncarnation::from_recorded(incarnation_owner, 3),
                    generation,
                    DurableCompositeCommitIdentity::from_recorded(commit_owner, 9),
                ),
                Err(DurableProductReferenceOwnerMismatch)
            );
        }
        let reference = DurableProductBranchReference::from_recorded(
            branch.clone(),
            DurableProductBranchIncarnation::from_recorded(first, 3),
            generation,
            DurableCompositeCommitIdentity::from_recorded(first, 9),
        )
        .unwrap();
        assert_eq!(reference.branch(), &branch);
        assert_eq!(reference.generation().get(), 7);
        assert_eq!(reference.selected_commit().ordinal(), 9);
    }
}
