//! Reading, scripting and damaging the test medium's branch transitions.

use super::{ScriptedPublication, TestDurableMedium};
use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;

impl TestDurableMedium {
    /// The last transition sequence this medium made durable, or zero.
    pub(crate) fn transition_head(&self) -> RelationalBranchTransitionSequence {
        self.transition_journal().head()
    }

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

    /// Flip every bit of one byte of the transition held at `sequence`.
    pub(crate) fn flip_transition_byte(
        &self,
        sequence: RelationalBranchTransitionSequence,
        offset: usize,
    ) {
        let mut journal = self.transition_journal();
        let bytes = journal
            .bytes_at_mut(sequence)
            .unwrap_or_else(|| panic!("medium holds no transition at {sequence:?}"));
        bytes[offset] ^= 0xff;
    }

    /// Hold `bytes` at `sequence`, as storage that returns bytes Relational
    /// did not write there would.
    pub(crate) fn replace_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
        bytes: Box<[u8]>,
    ) {
        self.transition_journal().replace(sequence, bytes);
    }

    /// Drop the transition held at `sequence`, as a medium that lost it would.
    pub(crate) fn remove_transition(&self, sequence: RelationalBranchTransitionSequence) {
        self.transition_journal()
            .remove(sequence)
            .unwrap_or_else(|| panic!("medium holds no transition at {sequence:?}"));
    }
}
