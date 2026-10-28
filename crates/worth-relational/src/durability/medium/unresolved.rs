use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifactDigest;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactDigest;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;

/// A durable publication that neither completed nor was proven to have had
/// no effect.
///
/// Nothing was installed into live state. The fate is decided by the
/// medium's recovery: after reopening, the subject is durable exactly when
/// the recovered history holds it with this digest. Until the runtime is
/// reopened it refuses every later commit and every later branch transition
/// with the same answer: a later commit must not overtake an unresolved
/// delete, and a later transition must not overtake an unresolved commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalDurabilityUnresolved {
    subject: RelationalDurabilityUnresolvedSubject,
    detail: String,
}

/// What an unresolved publication would have made durable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalDurabilityUnresolvedSubject {
    /// A canonical commit at its patch-stream position.
    CanonicalCommit {
        position: PatchStreamPosition,
        commit_id: CommitId,
        digest: RelationalCanonicalCommitArtifactDigest,
    },
    /// A branch transition at its sequence, after the position it follows.
    BranchTransition {
        sequence: RelationalBranchTransitionSequence,
        follows: PatchStreamPosition,
        digest: RelationalBranchTransitionArtifactDigest,
    },
}

impl RelationalDurabilityUnresolved {
    pub(crate) fn new(subject: RelationalDurabilityUnresolvedSubject, detail: String) -> Self {
        Self { subject, detail }
    }

    /// The publication whose fate is unresolved, with the digest that
    /// resolves it.
    pub fn subject(&self) -> RelationalDurabilityUnresolvedSubject {
        self.subject
    }

    /// The medium's description of why the fate is unknown.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for RelationalDurabilityUnresolvedSubject {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CanonicalCommit {
                position,
                commit_id,
                ..
            } => write!(
                formatter,
                "canonical commit {} at position {}",
                commit_id.0, position.0
            ),
            Self::BranchTransition {
                sequence, follows, ..
            } => write!(
                formatter,
                "branch transition {} following position {}",
                sequence.0, follows.0
            ),
        }
    }
}
