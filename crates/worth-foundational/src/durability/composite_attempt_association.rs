use super::{
    identity_seal, DurableCompositePublicationAttemptIdentity, DurableIdentity,
    DurableProductBranchIdentity, DurableProductBranchIncarnation,
};
/// The complete owner-neutral association of an effect with a World attempt.
/// Product incarnation is branch lifecycle data, never the Home authority fence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeAttemptAssociation {
    attempt: DurableCompositePublicationAttemptIdentity,
    branch: DurableProductBranchIdentity,
    incarnation: DurableProductBranchIncarnation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompositeAttemptAssociationOwnerMismatch;
impl CompositeAttemptAssociation {
    pub fn from_recorded(
        attempt: DurableCompositePublicationAttemptIdentity,
        branch: DurableProductBranchIdentity,
        incarnation: DurableProductBranchIncarnation,
    ) -> Result<Self, CompositeAttemptAssociationOwnerMismatch> {
        if attempt.owner() != branch.owner() || attempt.owner() != incarnation.owner() {
            return Err(CompositeAttemptAssociationOwnerMismatch);
        }
        Ok(Self {
            attempt,
            branch,
            incarnation,
        })
    }
    pub const fn attempt(&self) -> DurableCompositePublicationAttemptIdentity {
        self.attempt
    }
    pub fn branch(&self) -> &DurableProductBranchIdentity {
        &self.branch
    }
    pub const fn incarnation(&self) -> DurableProductBranchIncarnation {
        self.incarnation
    }
}
impl identity_seal::Sealed for CompositeAttemptAssociation {}
impl DurableIdentity for CompositeAttemptAssociation {}
