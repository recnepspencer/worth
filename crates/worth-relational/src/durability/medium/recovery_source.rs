use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::RelationalNativeCheckpoint;
use crate::publication::patch::data::PatchStreamPosition;

/// The durable checkpoint a medium selected, with the coverage its owner
/// recorded when it was published: the last commit position and the last
/// branch transition sequence it includes.
///
/// Recovery requires the checkpoint's own coverage to equal both, so a
/// checkpoint filed under the wrong coverage is refused, never reinterpreted.
#[derive(Debug, Clone)]
pub struct RelationalDurableCheckpointSource {
    covered: PatchStreamPosition,
    covered_transitions: RelationalBranchTransitionSequence,
    checkpoint: RelationalNativeCheckpoint,
}

impl RelationalDurableCheckpointSource {
    pub fn new(
        covered: PatchStreamPosition,
        covered_transitions: RelationalBranchTransitionSequence,
        checkpoint: RelationalNativeCheckpoint,
    ) -> Self {
        Self {
            covered,
            covered_transitions,
            checkpoint,
        }
    }

    pub fn covered(&self) -> PatchStreamPosition {
        self.covered
    }

    pub fn covered_transitions(&self) -> RelationalBranchTransitionSequence {
        self.covered_transitions
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
/// `tail` holds every durable commit position after the checkpoint's
/// coverage (or from position one without a checkpoint), in ascending order.
/// `transitions` holds every durable branch transition after the
/// checkpoint's transition coverage (or from sequence one), in ascending
/// order. Relational readmits all of it before any of it becomes authority.
#[derive(Debug, Clone)]
pub struct RelationalDurableRecoverySource {
    checkpoint: Option<RelationalDurableCheckpointSource>,
    tail: Vec<RelationalCanonicalCommitArtifact>,
    transitions: Vec<RelationalBranchTransitionArtifact>,
}

impl RelationalDurableRecoverySource {
    pub fn new(
        checkpoint: Option<RelationalDurableCheckpointSource>,
        tail: Vec<RelationalCanonicalCommitArtifact>,
        transitions: Vec<RelationalBranchTransitionArtifact>,
    ) -> Self {
        Self {
            checkpoint,
            tail,
            transitions,
        }
    }

    pub fn checkpoint(&self) -> Option<&RelationalDurableCheckpointSource> {
        self.checkpoint.as_ref()
    }

    pub fn tail(&self) -> &[RelationalCanonicalCommitArtifact] {
        &self.tail
    }

    pub fn transitions(&self) -> &[RelationalBranchTransitionArtifact] {
        &self.transitions
    }

    pub(crate) fn into_parts(self) -> RecoverySourceParts {
        RecoverySourceParts {
            checkpoint: self.checkpoint,
            tail: self.tail,
            transitions: self.transitions,
        }
    }
}

/// A recovery source taken apart for readmission.
pub(crate) struct RecoverySourceParts {
    pub(crate) checkpoint: Option<RelationalDurableCheckpointSource>,
    pub(crate) tail: Vec<RelationalCanonicalCommitArtifact>,
    pub(crate) transitions: Vec<RelationalBranchTransitionArtifact>,
}
