//! Original positioned transition accepted by its bound medium.
use super::PositionedBranchTransition;

#[derive(Debug)]
pub struct RelationalDurableTransitionPublication {
    positioned: PositionedBranchTransition,
    digest: [u8; 32],
}

impl worth_proof::AuthorityMarker for RelationalDurableTransitionPublication {}

impl RelationalDurableTransitionPublication {
    pub fn accepted(positioned: PositionedBranchTransition, digest: [u8; 32]) -> Self {
        Self {
            positioned: positioned.with_recorded_artifact_digest(Some(digest)),
            digest,
        }
    }

    pub(super) fn into_evidence(self) -> (PositionedBranchTransition, [u8; 32]) {
        (self.positioned, self.digest)
    }
}
