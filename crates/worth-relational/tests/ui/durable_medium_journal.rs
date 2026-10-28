//! A construction-only medium for the ownership compile courts.
use std::sync::Arc;
use worth_relational::facade::{
    durability::*, storage::RelationalStorageUnavailableCause as Cause,
};
#[derive(Debug)]
pub struct Journal;
impl RelationalDurableMedium for Journal {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        Arc::new(Journal)
    }
    fn read_transition(
        &self,
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        Err(RelationalDurableTransitionReadFailure::Missing(sequence))
    }
    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> RelationalDurablePublication {
        RelationalDurablePublication::Committed
    }
    fn publish_transition(
        &self,
        _: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        RelationalDurablePublication::Committed
    }
}
impl RelationalDurableColdSource for Journal {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
    fn read_commit(
        &self,
        _: worth_relational::facade::publication::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}
