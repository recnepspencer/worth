mod aspect_history;
mod branch_creation;
mod canonical_commit_allocation;
mod canonical_commit_envelope;
mod canonical_record_allocation;
mod committed_version;
mod merge_branch_basis;
#[cfg(test)]
mod merge_branch_basis_foundational;
mod positioned_canonical_commit;

use serde::{Deserialize, Serialize};

use crate::identity::data::VersionId;
use crate::identity::data::{EntityId, RelationId};

pub use aspect_history::{
    AspectHistoryCommitSpan, AspectHistoryDigest, AspectHistoryEntry,
    AspectHistoryLineageEventSpan, AspectHistoryOrigin, AspectHistoryQueryResult,
    AspectHistoryResolutionTrace, AspectResolutionContext, HistoryAspectQueryTarget,
    LineageAspectHistory, LineageAspectHistoryQueryResult, LineageAspectResolutionDigest,
};
pub use branch_creation::{BranchCreateError, BranchCreateErrorClass};
pub use canonical_commit_envelope::{
    CanonicalCommitAuthorityKind, CanonicalCommitEnvelope, RelationalReplayRecord,
    ReplaySchemaVersion,
};
pub(crate) use canonical_commit_envelope::{CheckpointCanonicalEnvelopeRef, CommittedRecordChange};
pub use canonical_record_allocation::RecordAllocationClass;
pub(crate) use canonical_record_allocation::{CanonicalRecordAllocation, RecordAllocationOrigin};
pub use committed_version::CommittedVersionSummary;
pub use merge_branch_basis::{
    MergeBaseSelectionRule, RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial,
    ResolvedMergeBase,
};
#[cfg(test)]
pub use merge_branch_basis_foundational::{
    RelationalFoundationalCurrentMergeBranchBasisArtifact,
    RelationalMergeBranchBasisFoundationalLoweringDenial,
};
pub(crate) use positioned_canonical_commit::PositionedCanonicalCommit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BranchId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionGraphPolicy {
    CanonicalSerializedPublication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryRetentionClass {
    Ephemeral,
    Durable,
    AuditGrade,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Immutable commit receipt. The branch id is authoring provenance only; it
/// never represents or selects a mutable branch head.
pub struct RelationalCommitReceipt {
    pub commit_id: CommitId,
    pub version_id: VersionId,
    pub branch_id: BranchId,
    pub parents: Vec<CommitId>,
}

/// Consumer-side semantic wrapper for authoritative parent order.
///
/// `RelationalCommitReceipt.parents` remains the sole authoritative storage and
/// publication surface. This wrapper exists only to make order-sensitive
/// consumption explicit in parity, certification, and diagnostics paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderedParentList {
    parents: Vec<CommitId>,
}

/// Intentionally coarse history-shape classification for 7A assumption
/// removal, diagnostics wording, and certification branching.
///
/// This is not a merge semantic policy model. Future merge execution work may
/// need finer distinctions than this enum provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryShapeClassification {
    Root,
    Linear,
    MergeReady,
}

/// Typed 7A history drift taxonomy for authoritative ordered-parent parity.
///
/// These variants classify how canonical history truth diverged; they do not
/// replace replay or durability failure classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryDriftClass {
    CanonicalHistoryDrift,
    ReplayAuthorityDrift,
    DurabilityParityDrift,
}

impl RelationalCommitReceipt {
    pub fn ordered_parents(&self) -> OrderedParentList {
        OrderedParentList::from_authoritative(self.parents.clone())
    }

    pub fn history_shape_classification(&self) -> HistoryShapeClassification {
        HistoryShapeClassification::from_parent_count(self.parents.len())
    }
}

impl OrderedParentList {
    pub fn from_authoritative(parents: Vec<CommitId>) -> Self {
        Self { parents }
    }

    pub fn as_slice(&self) -> &[CommitId] {
        &self.parents
    }

    pub fn len(&self) -> usize {
        self.parents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }

    pub fn clone_inner(&self) -> Vec<CommitId> {
        self.parents.clone()
    }

    pub fn history_shape_classification(&self) -> HistoryShapeClassification {
        HistoryShapeClassification::from_parent_count(self.parents.len())
    }
}

impl HistoryShapeClassification {
    pub fn from_parent_count(parent_count: usize) -> Self {
        match parent_count {
            0 => Self::Root,
            1 => Self::Linear,
            _ => Self::MergeReady,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VersionNode {
    pub(crate) commit: RelationalCommitReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MergeConflictRecord {
    Entity(EntityId),
    Relation(RelationId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeInspection {
    pub source_branch: BranchId,
    pub target_branch: BranchId,
    pub source_head: Option<RelationalCommitReceipt>,
    pub target_head: Option<RelationalCommitReceipt>,
    /// Current merge-base result under the runtime's common-ancestor selection
    /// rule.
    pub merge_base: Option<CommitId>,
    /// Branch-local ancestor closure for the source head after removing the
    /// merge-base ancestor closure, ordered by ascending `CommitId`.
    pub source_only_commits: Vec<CommitId>,
    /// Branch-local ancestor closure for the target head after removing the
    /// merge-base ancestor closure, ordered by ascending `CommitId`.
    pub target_only_commits: Vec<CommitId>,
    pub conflicting_records: Vec<MergeConflictRecord>,
    pub can_merge: bool,
}
