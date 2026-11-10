//! The test medium's durable contents and its position rule.

use std::collections::VecDeque;
use worth_testkit_relational::durable_medium::bytes::ByteJournal;

use super::ScriptedPublication;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{DurablePublicationFate, RelationalDurableCheckpointSource};
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Debug, Default)]
pub(super) struct TestDurableJournal {
    pub(super) initial_inputs:
        Option<crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact>,
    pub(super) selected_capture: Option<crate::durability::medium::OpaqueCapturedCheckpointSubject>,
    pub(super) artifacts: ByteJournal,
    pub(super) checkpoint_members: worth_testkit_relational::durable_medium::bytes::MemberBytes,
    pub(super) checkpoint: Option<RelationalDurableCheckpointSource>,
    pub(super) scripted: VecDeque<ScriptedPublication>,
    pub(super) offered: Vec<RelationalCanonicalCommitArtifact>,
}

impl TestDurableJournal {
    pub(super) fn file_checkpoint(&mut self, checkpoint: RelationalDurableCheckpointSource) {
        for member in checkpoint.checkpoint().members() {
            if let crate::durability::data::RelationalCheckpointMember::New { locator, bytes } =
                member
            {
                self.checkpoint_members
                    .insert(locator.sha256, locator.length, bytes.clone());
            }
        }
        self.selected_capture = checkpoint.checkpoint().publication_subject().cloned();
        self.checkpoint = Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered(),
            checkpoint.covered_transitions(),
            crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes(
                checkpoint.checkpoint().bytes().to_vec(),
            ),
        ));
    }

    pub(super) fn member_bytes(
        &self,
        locator: crate::durability::medium::RelationalDurableMemberLocator,
    ) -> Option<&[u8]> {
        self.checkpoint_members.get(locator.sha256, locator.length)
    }

    pub(super) fn head(&self) -> PatchStreamPosition {
        PatchStreamPosition(
            self.artifacts
                .entries
                .keys()
                .next_back()
                .copied()
                .unwrap_or(0),
        )
    }

    pub(super) fn positions(&self) -> Vec<PatchStreamPosition> {
        self.artifacts
            .entries
            .keys()
            .copied()
            .map(PatchStreamPosition)
            .collect()
    }

    pub(super) fn bytes_at(&self, position: PatchStreamPosition) -> Option<&[u8]> {
        self.artifacts.entries.get(&position.0).map(AsRef::as_ref)
    }

    pub(super) fn replace(&mut self, position: PatchStreamPosition, bytes: Box<[u8]>) {
        self.artifacts.entries.insert(position.0, bytes);
    }

    pub(super) fn artifacts_after(
        &self,
        covered: PatchStreamPosition,
    ) -> impl Iterator<Item = &[u8]> {
        self.artifacts
            .entries
            .range(covered.0.saturating_add(1)..)
            .map(|(_, bytes)| bytes.as_ref())
    }

    /// The medium's ordinary rule: the next position is durable, an
    /// identical retry of a durable position is durable, anything else had
    /// no effect.
    pub(super) fn publish_at_head(
        &mut self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        let position = artifact.position();
        if self.artifacts.publish(position.0, artifact.as_bytes()) {
            return DurablePublicationFate::Committed;
        }
        DurablePublicationFate::ProvenNoEffect {
            detail: format!(
                "position {} is not the next durable position {}",
                position.0,
                self.head().0 + 1
            ),
        }
    }

    pub(super) fn is_next(&self, position: PatchStreamPosition) -> bool {
        position.0 == self.head().0 + 1
    }

    pub(super) fn store(&mut self, artifact: &RelationalCanonicalCommitArtifact) {
        self.artifacts
            .entries
            .insert(artifact.position().0, Box::from(artifact.as_bytes()));
    }
}
