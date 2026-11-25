use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::{BranchId, CommitId};
use crate::history::data::merge_branch_basis::MergeBaseSelectionRule;
use crate::identity::data::VersionId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDeltaSummary {
    pub branch_id: BranchId,
    pub head_commit_id: CommitId,
    pub head_version_id: VersionId,
    pub unique_commit_count: usize,
    pub touched_record_count: usize,
    pub touched_entity_count: usize,
    pub touched_relation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeAncestrySummary {
    pub merge_base_rule: MergeBaseSelectionRule,
    pub merge_base_commit_id: CommitId,
    pub supporting_left_ancestor_count: usize,
    pub supporting_right_ancestor_count: usize,
    pub target: BranchDeltaSummary,
    pub source: BranchDeltaSummary,
}
