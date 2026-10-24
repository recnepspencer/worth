//! Damage the test medium's durable contents, standing in for storage that
//! returns bytes Relational did not write.

use std::sync::Arc;

use super::TestDurableMedium;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

/// Offset of the artifact header's commit-id field. The payload digest does
/// not cover the header, so damage here passes the frame check and must be
/// refused by readmission.
pub(crate) const ARTIFACT_HEADER_COMMIT_ID_OFFSET: usize = 18;

impl TestDurableMedium {
    /// Flip every bit of one byte of the artifact held at `position`.
    pub(crate) fn flip_artifact_byte(&self, position: PatchStreamPosition, offset: usize) {
        let mut journal = self.journal();
        let bytes = journal
            .bytes_at_mut(position)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
        bytes[offset] ^= 0xff;
    }

    /// Flip every bit of one byte of the filed checkpoint, keeping the
    /// position it is filed under.
    pub(crate) fn flip_checkpoint_byte(&self, offset: usize) {
        let mut journal = self.journal();
        let filed = journal
            .checkpoint
            .take()
            .expect("medium holds a checkpoint");
        let mut bytes = filed.checkpoint().bytes().to_vec();
        bytes[offset] ^= 0xff;
        journal.checkpoint = Some(
            crate::durability::medium::RelationalDurableCheckpointSource::new(
                filed.covered(),
                RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
            ),
        );
    }

    /// Damage the filed checkpoint's payload so it no longer decodes: the
    /// first byte after the frame magic is the payload's map marker.
    pub(crate) fn damage_checkpoint_payload(&self) {
        self.flip_checkpoint_byte(
            crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC.len(),
        );
    }

    /// Lose the artifact at `position`, leaving a gap in the journal.
    pub(crate) fn drop_artifact(&self, position: PatchStreamPosition) {
        self.journal()
            .remove(position)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
    }

    /// Hold the artifact from `from` again at position `at`, as a medium that
    /// duplicated an earlier entry would.
    pub(crate) fn duplicate_artifact(&self, from: PatchStreamPosition, at: PatchStreamPosition) {
        let mut journal = self.journal();
        let bytes = Box::from(
            journal
                .bytes_at(from)
                .unwrap_or_else(|| panic!("medium holds no artifact at {from:?}")),
        );
        journal.replace(at, bytes);
    }

    /// Rewrite the envelope of the durable commit `commit_id`, wherever the
    /// medium holds it. See [`Self::rewrite_artifact_envelope`].
    pub(crate) fn rewrite_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: crate::history::data::CommitId,
        edit: impl FnOnce(&mut CanonicalCommitEnvelope),
    ) {
        let position = self
            .positions()
            .into_iter()
            .find(|position| {
                let bytes = self.artifact_bytes(*position).expect("held position");
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes)
                    .is_ok_and(|artifact| artifact.commit_id() == commit_id)
            })
            .unwrap_or_else(|| panic!("medium holds no artifact for {commit_id:?}"));
        self.rewrite_artifact_envelope(execution, position, edit);
    }

    /// Replace the artifact at `position` with a well-formed artifact whose
    /// envelope `edit` has rewritten: storage that kept a valid frame around
    /// commit meaning Relational never published.
    pub(crate) fn rewrite_artifact_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        position: PatchStreamPosition,
        edit: impl FnOnce(&mut CanonicalCommitEnvelope),
    ) {
        let mut journal = self.journal();
        let readmitted = RelationalCanonicalCommitArtifact::readmit(
            journal
                .bytes_at(position)
                .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}")),
            execution,
        )
        .expect("the medium's own artifact readmits before it is rewritten");
        let mut envelope = readmitted.envelope().clone();
        edit(&mut envelope);
        let rewritten = RelationalCanonicalCommitArtifact::from_positioned(
            &PositionedCanonicalCommit::for_test(position, Arc::new(envelope)),
            execution.allocation_policy(),
        )
        .expect("a rewritten envelope still encodes");
        journal.replace(position, Box::from(rewritten.as_bytes()));
    }
}
