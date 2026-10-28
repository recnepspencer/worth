//! The test medium's branch transition journal: transition artifacts by
//! sequence, with the same rule as the commit journal (the next sequence, or
//! an identical retry) and its own scripted answers.

use std::collections::{BTreeMap, VecDeque};

use super::ScriptedPublication;
use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::medium::RelationalDurablePublication;

#[derive(Debug, Default)]
pub(super) struct TestTransitionJournal {
    artifacts: BTreeMap<u64, Box<[u8]>>,
    pub(super) scripted: VecDeque<ScriptedPublication>,
    pub(super) offered: Vec<RelationalBranchTransitionArtifact>,
}

impl TestTransitionJournal {
    pub(super) fn head(&self) -> RelationalBranchTransitionSequence {
        RelationalBranchTransitionSequence(self.artifacts.keys().next_back().copied().unwrap_or(0))
    }

    pub(super) fn sequences(&self) -> Vec<RelationalBranchTransitionSequence> {
        self.artifacts
            .keys()
            .copied()
            .map(RelationalBranchTransitionSequence)
            .collect()
    }

    pub(super) fn bytes_at(&self, sequence: RelationalBranchTransitionSequence) -> Option<&[u8]> {
        self.artifacts.get(&sequence.0).map(AsRef::as_ref)
    }

    pub(super) fn bytes_at_mut(
        &mut self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Option<&mut [u8]> {
        self.artifacts.get_mut(&sequence.0).map(AsMut::as_mut)
    }

    pub(super) fn replace(
        &mut self,
        sequence: RelationalBranchTransitionSequence,
        bytes: Box<[u8]>,
    ) {
        self.artifacts.insert(sequence.0, bytes);
    }

    pub(super) fn remove(
        &mut self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Option<Box<[u8]>> {
        self.artifacts.remove(&sequence.0)
    }

    pub(super) fn artifacts_after(
        &self,
        covered: RelationalBranchTransitionSequence,
    ) -> impl Iterator<Item = &[u8]> {
        self.artifacts
            .range(covered.0.saturating_add(1)..)
            .map(|(_, bytes)| bytes.as_ref())
    }

    /// Answer one transition publication: as scripted, otherwise by sequence.
    pub(super) fn publish(
        &mut self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        self.offered.push(artifact.clone());
        match self.scripted.pop_front() {
            Some(answer) => answer.answer_transition(self, artifact),
            None => self.publish_at_head(artifact),
        }
    }

    fn publish_at_head(
        &mut self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        let sequence = artifact.sequence();
        if self.is_next(sequence) {
            self.store(artifact);
            return RelationalDurablePublication::Committed;
        }
        if self.bytes_at(sequence) == Some(artifact.as_bytes()) {
            return RelationalDurablePublication::Committed;
        }
        RelationalDurablePublication::ProvenNoEffect {
            detail: format!(
                "sequence {} is not the next durable sequence {}",
                sequence.0,
                self.head().0 + 1
            ),
        }
    }

    pub(super) fn is_next(&self, sequence: RelationalBranchTransitionSequence) -> bool {
        sequence.0 == self.head().0 + 1
    }

    pub(super) fn store(&mut self, artifact: &RelationalBranchTransitionArtifact) {
        self.artifacts
            .insert(artifact.sequence().0, Box::from(artifact.as_bytes()));
    }
}
