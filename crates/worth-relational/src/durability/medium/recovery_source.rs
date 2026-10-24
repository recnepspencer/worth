use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::publication::patch::data::PatchStreamPosition;

/// The durable checkpoint a medium selected, with the position its owner
/// recorded as covered when it was published.
///
/// Recovery requires the checkpoint's own coverage to equal `covered`, so a
/// checkpoint filed under the wrong position is refused, never reinterpreted.
#[derive(Debug, Clone)]
pub struct RelationalDurableCheckpointSource {
    covered: PatchStreamPosition,
    checkpoint: RelationalNativeCheckpoint,
}

impl RelationalDurableCheckpointSource {
    pub fn new(covered: PatchStreamPosition, checkpoint: RelationalNativeCheckpoint) -> Self {
        Self {
            covered,
            checkpoint,
        }
    }

    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }

    pub fn checkpoint(&self) -> &RelationalNativeCheckpoint {
        &self.checkpoint
    }

    pub(crate) fn into_checkpoint(self) -> RelationalNativeCheckpoint {
        self.checkpoint
    }
}

/// Everything a medium recovered for one Relational home, as untrusted bytes.
///
/// `tail` holds every durable position after the checkpoint's coverage (or
/// from position one without a checkpoint), in ascending order. Relational
/// readmits all of it before any of it becomes authority.
#[derive(Debug, Clone)]
pub struct RelationalDurableRecoverySource {
    checkpoint: Option<RelationalDurableCheckpointSource>,
    tail: Vec<RelationalCanonicalCommitArtifact>,
}

impl RelationalDurableRecoverySource {
    pub fn new(
        checkpoint: Option<RelationalDurableCheckpointSource>,
        tail: Vec<RelationalCanonicalCommitArtifact>,
    ) -> Self {
        Self { checkpoint, tail }
    }

    pub fn checkpoint(&self) -> Option<&RelationalDurableCheckpointSource> {
        self.checkpoint.as_ref()
    }

    pub fn tail(&self) -> &[RelationalCanonicalCommitArtifact] {
        &self.tail
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<RelationalDurableCheckpointSource>,
        Vec<RelationalCanonicalCommitArtifact>,
    ) {
        (self.checkpoint, self.tail)
    }
}
