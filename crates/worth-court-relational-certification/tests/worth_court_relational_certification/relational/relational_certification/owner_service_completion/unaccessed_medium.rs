//! A genuine attached medium whose data/effect doors must never be contacted.
use std::sync::Arc;
use worth_relational::facade::{durability::*, storage::RelationalStorageUnavailableCause};

#[derive(Debug)]
pub(super) struct UnaccessedMedium;
impl RelationalDurableColdSource for UnaccessedMedium {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        panic!("unsupported profile must not acquire medium members")
    }
    fn read_member(
        &self,
        _: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        panic!("unsupported profile must not read medium members")
    }
    fn read_commit(
        &self,
        _: worth_relational::facade::publication::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        panic!("unsupported profile must not read medium commits")
    }
}
impl RelationalDurableMedium for UnaccessedMedium {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        Arc::new(Self)
    }
    fn read_transition(
        &self,
        _: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        panic!("unsupported profile must not read medium transitions")
    }
    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> RelationalDurablePublication {
        panic!("unsupported profile must not publish medium commits")
    }
    fn publish_transition(
        &self,
        _: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        panic!("unsupported profile must not publish medium transitions")
    }
}
