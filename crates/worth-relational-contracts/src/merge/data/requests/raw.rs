use serde::{Deserialize, Serialize};

use crate::history::data::coordinates::BranchId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeIntent {
    ReconcileIntoTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Descriptive merge selectors. These ids are not branch-head authority:
/// owner planning resolves them to the exact branch-cell observation and
/// truth version, and prepared execution verifies both again before effects.
pub struct MergePlanningRequest {
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
}

impl MergePlanningRequest {
    pub fn new(
        target_branch: BranchId,
        source_branch: BranchId,
        merge_intent: MergeIntent,
    ) -> Self {
        Self {
            target_branch,
            source_branch,
            merge_intent,
        }
    }

    pub fn target_branch(&self) -> &BranchId {
        &self.target_branch
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    pub fn merge_intent(&self) -> MergeIntent {
        self.merge_intent
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Descriptive execution selectors. They carry no expected commit or
/// currentness claim; the owner binds the request to exact branch references
/// during preparation and rejects any reference movement before publication.
pub struct MergeExecutionRequest {
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
}

impl MergeExecutionRequest {
    pub fn new(
        target_branch: BranchId,
        source_branch: BranchId,
        merge_intent: MergeIntent,
    ) -> Self {
        Self {
            target_branch,
            source_branch,
            merge_intent,
        }
    }

    pub fn target_branch(&self) -> &BranchId {
        &self.target_branch
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    pub fn merge_intent(&self) -> MergeIntent {
        self.merge_intent
    }
}

impl From<MergeExecutionRequest> for MergePlanningRequest {
    fn from(value: MergeExecutionRequest) -> Self {
        Self {
            target_branch: value.target_branch,
            source_branch: value.source_branch,
            merge_intent: value.merge_intent,
        }
    }
}

impl From<MergePlanningRequest> for MergeExecutionRequest {
    fn from(value: MergePlanningRequest) -> Self {
        Self {
            target_branch: value.target_branch,
            source_branch: value.source_branch,
            merge_intent: value.merge_intent,
        }
    }
}
