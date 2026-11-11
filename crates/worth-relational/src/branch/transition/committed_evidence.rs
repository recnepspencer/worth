//! Original ACK evidence leaves Native only after its real transition cutover.
use super::PositionedBranchTransition;
use crate::durability::medium::RelationalDurableTransitionPublication;

/// An owner-issued original subject, not decodable permission or an installed-target proxy.
#[derive(Debug)]
pub struct RelationalCommittedTransitionEvidence {
    positioned: std::sync::Arc<PositionedBranchTransition>,
    digest: [u8; 32],
}
impl worth_proof::AuthorityMarker for RelationalCommittedTransitionEvidence {}

impl RelationalCommittedTransitionEvidence {
    pub(crate) fn from_publication(
        publication: RelationalDurableTransitionPublication,
        retain: impl FnOnce(PositionedBranchTransition) -> std::sync::Arc<PositionedBranchTransition>,
    ) -> Self {
        let (positioned, digest) = publication.into_evidence();
        Self {
            positioned: retain(positioned),
            digest,
        }
    }

    pub(crate) fn retained_positioned(&self) -> std::sync::Arc<PositionedBranchTransition> {
        std::sync::Arc::clone(&self.positioned)
    }

    pub fn positioned(&self) -> &PositionedBranchTransition {
        &self.positioned
    }

    pub fn artifact_digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
