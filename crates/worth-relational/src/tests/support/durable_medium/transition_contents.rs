//! Reading, scripting and damaging the test medium's branch transitions.

use super::{ScriptedPublication, TestDurableMedium};
use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;

impl TestDurableMedium {
    /// Every transition sequence this medium holds, ascending.
    pub(crate) fn transition_sequences(&self) -> Vec<RelationalBranchTransitionSequence> {
        self.transition_journal().sequences()
    }

    /// The exact transition bytes held at `sequence`.
    pub(crate) fn transition_bytes(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Option<Box<[u8]>> {
        self.transition_journal().bytes_at(sequence).map(Box::from)
    }

    /// Every transition artifact a publication offered, in order, whatever
    /// the answer was.
    pub(crate) fn offered_transitions(&self) -> Vec<RelationalBranchTransitionArtifact> {
        self.transition_journal().offered.clone()
    }

    /// Answer the next transition publication as scripted instead of by
    /// sequence.
    pub(crate) fn script_next_transition(&self, answer: ScriptedPublication) {
        self.transition_journal().scripted.push_back(answer);
    }
}
