use super::coordinates::{BranchId, CommitId};
use super::history_policy::HistoryShapeClassification;
use crate::identity::data::VersionId;
use serde::{Deserialize, Serialize};

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
