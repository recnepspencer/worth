//! The test medium's durable contents and its position rule.

use std::collections::{BTreeMap, VecDeque};

use super::ScriptedPublication;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::medium::{RelationalDurableCheckpointSource, RelationalDurablePublication};
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Debug, Default)]
pub(super) struct TestDurableJournal {
    artifacts: BTreeMap<u64, Box<[u8]>>,
    pub(super) checkpoint: Option<RelationalDurableCheckpointSource>,
    pub(super) scripted: VecDeque<ScriptedPublication>,
    pub(super) offered: Vec<RelationalCanonicalCommitArtifact>,
}

impl TestDurableJournal {
    pub(super) fn head(&self) -> PatchStreamPosition {
        PatchStreamPosition(self.artifacts.keys().next_back().copied().unwrap_or(0))
    }

    pub(super) fn positions(&self) -> Vec<PatchStreamPosition> {
        self.artifacts
            .keys()
            .copied()
            .map(PatchStreamPosition)
            .collect()
    }

    pub(super) fn bytes_at(&self, position: PatchStreamPosition) -> Option<&[u8]> {
        self.artifacts.get(&position.0).map(AsRef::as_ref)
    }

    pub(super) fn bytes_at_mut(&mut self, position: PatchStreamPosition) -> Option<&mut [u8]> {
        self.artifacts.get_mut(&position.0).map(AsMut::as_mut)
    }

    pub(super) fn replace(&mut self, position: PatchStreamPosition, bytes: Box<[u8]>) {
        self.artifacts.insert(position.0, bytes);
    }

    pub(super) fn remove(&mut self, position: PatchStreamPosition) -> Option<Box<[u8]>> {
        self.artifacts.remove(&position.0)
    }

    pub(super) fn artifacts_after(
        &self,
        covered: PatchStreamPosition,
    ) -> impl Iterator<Item = &[u8]> {
        self.artifacts
            .range(covered.0.saturating_add(1)..)
            .map(|(_, bytes)| bytes.as_ref())
    }

    /// The medium's ordinary rule: the next position is durable, an
    /// identical retry of a durable position is durable, anything else had
    /// no effect.
    pub(super) fn publish_at_head(
        &mut self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        let position = artifact.position();
        if self.is_next(position) {
            self.store(artifact);
            return RelationalDurablePublication::Committed;
        }
        if self.bytes_at(position) == Some(artifact.as_bytes()) {
            return RelationalDurablePublication::Committed;
        }
        RelationalDurablePublication::ProvenNoEffect {
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
            .insert(artifact.position().0, Box::from(artifact.as_bytes()));
    }
}
