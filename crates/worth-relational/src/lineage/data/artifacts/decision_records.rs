use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::identity::data::LineageId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageDecisionKind {
    CreateAccepted,
    ReplaceAccepted,
    RetireAccepted,
    SplitAccepted,
    MergeAccepted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageDecisionRecord {
    pub(crate) branch_id: BranchId,
    pub(crate) kind: LineageDecisionKind,
    pub(crate) event_id: Option<u64>,
    pub(crate) sources: Vec<LineageId>,
    pub(crate) targets: Vec<LineageId>,
}

impl LineageDecisionRecord {
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn kind(&self) -> &LineageDecisionKind {
        &self.kind
    }

    pub fn event_id(&self) -> Option<u64> {
        self.event_id
    }

    pub fn sources(&self) -> &[LineageId] {
        &self.sources
    }

    pub fn targets(&self) -> &[LineageId] {
        &self.targets
    }
}
