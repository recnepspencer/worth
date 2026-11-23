//! Pure-byte tamper and transition inspection helpers for `OwnedTestDurableMedium`.

use super::owned_medium::OwnedTestDurableMedium;
use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::data::{RelationalCheckpointMember, RelationalNativeCheckpoint};
use crate::durability::medium::RelationalDurableCheckpointSource;
use crate::publication::patch::data::PatchStreamPosition;

impl OwnedTestDurableMedium {
    pub(crate) fn flip_checkpoint_byte(&self, offset: usize) {
        let mut contents = self.lock();
        let filed = contents
            .checkpoint
            .take()
            .expect("medium holds a checkpoint");
        let mut bytes = filed.checkpoint().bytes().to_vec();
        bytes[offset] ^= 0xff;
        contents.checkpoint = Some(RelationalDurableCheckpointSource::new(
            filed.covered(),
            filed.covered_transitions(),
            RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
        ));
    }

    pub(crate) fn damage_checkpoint_payload(&self) {
        self.flip_checkpoint_byte(
            crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC.len(),
        );
    }

    pub(crate) fn file_checkpoint_under(
        &self,
        covered: PatchStreamPosition,
        covered_transitions: RelationalBranchTransitionSequence,
        checkpoint: RelationalNativeCheckpoint,
    ) {
        let mut contents = self.lock();
        for member in checkpoint.members() {
            if let RelationalCheckpointMember::New { locator, bytes } = member {
                contents
                    .members
                    .insert(locator.sha256, locator.length, bytes.clone());
            }
        }
        contents.selected = checkpoint.publication_subject().cloned();
        contents.checkpoint = Some(RelationalDurableCheckpointSource::new(
            covered,
            covered_transitions,
            RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        ));
    }

    pub(crate) fn flip_artifact_byte(&self, position: PatchStreamPosition, offset: usize) {
        let mut contents = self.lock();
        let bytes = contents
            .commits
            .entries
            .get_mut(&position.0)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
        bytes[offset] ^= 0xff;
    }

    pub(crate) fn corrupt_artifact_commit_header_with_intact_frame(
        &self,
        position: PatchStreamPosition,
    ) {
        let mut contents = self.lock();
        let bytes = contents
            .commits
            .entries
            .get_mut(&position.0)
            .expect("medium holds the original frame");
        bytes[18] ^= 0xff;
        super::reseal_canonical_commit_frame(bytes);
    }

    pub(crate) fn drop_artifact(&self, position: PatchStreamPosition) {
        self.lock()
            .commits
            .entries
            .remove(&position.0)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
    }

    pub(crate) fn duplicate_artifact(&self, from: PatchStreamPosition, at: PatchStreamPosition) {
        let mut contents = self.lock();
        let bytes = contents
            .commits
            .entries
            .get(&from.0)
            .cloned()
            .unwrap_or_else(|| panic!("medium holds no artifact at {from:?}"));
        contents.commits.entries.insert(at.0, bytes);
    }

    pub(crate) fn transition_head(&self) -> RelationalBranchTransitionSequence {
        RelationalBranchTransitionSequence(
            self.lock()
                .transitions
                .entries
                .keys()
                .next_back()
                .copied()
                .unwrap_or(0),
        )
    }

    pub(crate) fn transition_sequences(&self) -> Vec<RelationalBranchTransitionSequence> {
        self.lock()
            .transitions
            .entries
            .keys()
            .copied()
            .map(RelationalBranchTransitionSequence)
            .collect()
    }

    pub(crate) fn transition_bytes(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Option<Box<[u8]>> {
        self.lock().transitions.entries.get(&sequence.0).cloned()
    }

    pub(crate) fn flip_transition_byte(
        &self,
        sequence: RelationalBranchTransitionSequence,
        offset: usize,
    ) {
        let mut contents = self.lock();
        let bytes = contents
            .transitions
            .entries
            .get_mut(&sequence.0)
            .unwrap_or_else(|| panic!("medium holds no transition at {sequence:?}"));
        bytes[offset] ^= 0xff;
    }

    pub(crate) fn replace_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
        bytes: Box<[u8]>,
    ) {
        self.lock().transitions.entries.insert(sequence.0, bytes);
    }

    pub(crate) fn remove_transition(&self, sequence: RelationalBranchTransitionSequence) {
        self.lock()
            .transitions
            .entries
            .remove(&sequence.0)
            .unwrap_or_else(|| panic!("medium holds no transition at {sequence:?}"));
    }
}
