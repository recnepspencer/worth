//! The test medium's branch transition journal: transition artifacts by
//! sequence, with the same rule as the commit journal (the next sequence, or
//! an identical retry).

use worth_testkit_relational::durable_medium::bytes::ByteJournal;

use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::medium::DurablePublicationFate;

#[derive(Debug, Default)]
pub(super) struct TestTransitionJournal {
    pub(super) artifacts: ByteJournal,
}

impl TestTransitionJournal {
    pub(super) fn head(&self) -> RelationalBranchTransitionSequence {
        RelationalBranchTransitionSequence(
            self.artifacts
                .entries
                .keys()
                .next_back()
                .copied()
                .unwrap_or(0),
        )
    }

    pub(super) fn bytes_at(&self, sequence: RelationalBranchTransitionSequence) -> Option<&[u8]> {
        self.artifacts.entries.get(&sequence.0).map(AsRef::as_ref)
    }

    pub(super) fn artifacts_after(
        &self,
        covered: RelationalBranchTransitionSequence,
    ) -> impl Iterator<Item = &[u8]> {
        self.artifacts
            .entries
            .range(covered.0.saturating_add(1)..)
            .map(|(_, bytes)| bytes.as_ref())
    }

    /// Answer one transition publication by the ordinary sequence rule.
    pub(super) fn publish(
        &mut self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.publish_at_head(artifact)
    }

    fn publish_at_head(
        &mut self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        let sequence = artifact.sequence();
        if self.artifacts.publish(sequence.0, artifact.as_bytes()) {
            return DurablePublicationFate::Committed;
        }
        DurablePublicationFate::ProvenNoEffect {
            detail: format!(
                "sequence {} is not the next durable sequence {}",
                sequence.0,
                self.head().0 + 1
            ),
        }
    }
}
