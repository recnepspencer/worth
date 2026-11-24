//! Pure-byte tamper and transition inspection helpers for `OwnedTestDurableMedium`.

use super::owned_medium::OwnedTestDurableMedium;
use super::{Arc, CommitArt, Req};
use crate::branch::RelationalBranchTransitionSequence;
#[cfg(test)]
use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
use crate::durability::data::{RelationalCheckpointMember, RelationalNativeCheckpoint};
use crate::durability::published_medium::RelationalDurableCheckpointSource;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as StorageCause;
use sha2::{Digest, Sha256};
use std::sync::atomic::Ordering;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommitFixturePort as _;

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

    pub(crate) fn retain_offered_transition_wires(&self) {
        self.lock().retain_transition_wires = true;
    }

    pub(crate) fn start_recording_commit_reads(&self, position: PatchStreamPosition) {
        let mut contents = self.lock();
        let sha256: [u8; 32] = Sha256::digest(
            contents
                .commits
                .entries
                .get(&position.0)
                .unwrap_or_else(|| panic!("medium holds no commit at {position:?}")),
        )
        .into();
        contents.canonical_member_positions.insert(sha256, position);
        contents.commit_reads.clear();
        self.observe_commit_reads.store(true, Ordering::Release);
    }

    pub(crate) fn set_commit_read_failure(
        &self,
        position: PatchStreamPosition,
        cause: StorageCause,
    ) {
        let mut contents = self.lock();
        let sha256: [u8; 32] = Sha256::digest(
            contents
                .commits
                .entries
                .get(&position.0)
                .unwrap_or_else(|| panic!("medium holds no commit at {position:?}")),
        )
        .into();
        contents.canonical_member_positions.insert(sha256, position);
        contents.commit_read_failures.insert(position, cause);
        self.has_commit_read_failures.store(true, Ordering::Release);
    }

    pub(crate) fn clear_commit_read_failures(&self) {
        self.lock().commit_read_failures.clear();
        self.has_commit_read_failures
            .store(false, Ordering::Release);
    }

    pub(crate) fn commit_read_count(&self, position: PatchStreamPosition) -> usize {
        self.lock()
            .commit_reads
            .iter()
            .filter(|read| **read == position)
            .count()
    }

    pub(crate) fn artifact_sha256(&self, position: PatchStreamPosition) -> [u8; 32] {
        let bytes = self
            .artifact_bytes(position)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
        Sha256::digest(&bytes).into()
    }

    /// Remove the journal entry at `position` and make its protected checkpoint
    /// member unavailable so any unintended body read refuses as `MissingMember`.
    pub(crate) fn drop_artifact_and_member(&self, position: PatchStreamPosition) -> [u8; 32] {
        let mut contents = self.lock();
        let bytes = contents
            .commits
            .entries
            .remove(&position.0)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
        let sha256: [u8; 32] = Sha256::digest(&bytes).into();
        assert!(
            contents.members.get(sha256, bytes.len() as u64).is_some(),
            "checkpoint must hold the protected member for dropped position {position:?}"
        );
        contents.forbidden_members.insert(sha256);
        self.has_forbidden_members.store(true, Ordering::Release);
        sha256
    }

    pub(crate) fn start_recording_member_reads(&self) {
        self.lock().member_reads.clear();
        self.observe_member_reads.store(true, Ordering::Release);
    }

    pub(crate) fn member_read_digests(&self) -> Vec<[u8; 32]> {
        self.lock().member_reads.clone()
    }

    pub(super) fn check_and_record_member_read(
        &self,
        sha256: [u8; 32],
    ) -> Result<(), StorageCause> {
        let recording = self.observe_member_reads.load(Ordering::Acquire);
        let forbidding = self.has_forbidden_members.load(Ordering::Acquire);
        let recording_commits = self.observe_commit_reads.load(Ordering::Acquire);
        let failing_commits = self.has_commit_read_failures.load(Ordering::Acquire);
        if !recording && !forbidding && !recording_commits && !failing_commits {
            return Ok(());
        }
        let mut contents = self.lock();
        if recording {
            contents.member_reads.push(sha256);
        }
        if forbidding && contents.forbidden_members.contains(&sha256) {
            return Err(StorageCause::MissingMember);
        }
        if (recording_commits || failing_commits)
            && let Some(&position) = contents.canonical_member_positions.get(&sha256)
        {
            if recording_commits {
                contents.commit_reads.push(position);
            }
            if failing_commits && let Some(&cause) = contents.commit_read_failures.get(&position) {
                return Err(cause);
            }
        }
        Ok(())
    }

    pub(super) fn check_and_record_commit_read(
        &self,
        position: PatchStreamPosition,
    ) -> Result<(), StorageCause> {
        let recording = self.observe_commit_reads.load(Ordering::Acquire);
        let failing = self.has_commit_read_failures.load(Ordering::Acquire);
        if !recording && !failing {
            return Ok(());
        }
        let mut contents = self.lock();
        if recording {
            contents.commit_reads.push(position);
        }
        if failing && let Some(&cause) = contents.commit_read_failures.get(&position) {
            return Err(cause);
        }
        Ok(())
    }
}

impl OwnedTestDurableMedium {
    pub(crate) fn rewrite_artifact_envelope(
        &self,
        execution: &Req<'_, '_>,
        position: PatchStreamPosition,
        edit: impl FnOnce(&mut crate::history::data::CanonicalCommitEnvelopeBody),
    ) {
        let bytes = self
            .commit_bytes_at(position)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
        let readmitted = CommitArt::readmit(&bytes, execution)
            .expect("the medium's own artifact readmits before it is rewritten");
        let mut envelope = copy_original_for_test(readmitted.envelope(), execution);
        edit(envelope.body_mut_for_test());
        let rewritten = CommitArt::from_positioned(
            &crate::history::data::PositionedCanonicalCommit::positioned_commit_for_test(
                position,
                Arc::new(envelope),
            ),
            execution.allocation_policy(),
        )
        .expect("a rewritten envelope still encodes");
        self.replace_commit_bytes(position, Box::from(rewritten.as_bytes()));
    }
    pub(crate) fn rewrite_commit_envelope(
        &self,
        execution: &Req<'_, '_>,
        commit_id: crate::history::data::CommitId,
        edit: impl FnOnce(&mut crate::history::data::CanonicalCommitEnvelopeBody),
    ) {
        let position = self
            .positions()
            .into_iter()
            .find(|p| {
                self.artifact_bytes(*p)
                    .and_then(|b| CommitArt::from_untrusted_bytes(b).ok())
                    .is_some_and(|a| a.commit_id() == commit_id)
            })
            .unwrap_or_else(|| panic!("medium holds no artifact for {commit_id:?}"));
        self.rewrite_artifact_envelope(execution, position, edit);
    }
}
