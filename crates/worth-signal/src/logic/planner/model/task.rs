mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::OrderedStreamItem;
use crate::logic::evaluation::EvaluationRequestMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskReason {
    Dirty,
    MaybeStaleValidation,
    ConditionForced,
    RequestedTarget,
    DependencyRequired,
    MemoValidation,
    PartitionScopedDependency,
    OutputDiffDependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageBarrier {
    StageBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRecordId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemanticSegmentId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticTaskRange {
    pub start: ExecutionRecordId,
    pub end: ExecutionRecordId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateTask {
    pub node: NodeId,
    pub request_mode: EvaluationRequestMode,
    pub direct_request: bool,
    pub trigger_reason: TaskReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EligibleTask {
    pub node: NodeId,
    pub request_mode: EvaluationRequestMode,
    pub direct_request: bool,
    pub reason: TaskReason,
    pub admission: EligibleTaskAdmission,
}

impl OrderedStreamItem for EligibleTask {
    type OrderKey = (u32, u32);

    fn order_key(&self) -> Self::OrderKey {
        (self.node.index(), self.node.generation())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EligibleTaskAdmission {
    pub node_state_at_admission: Option<NodeState>,
    pub dirty_partition_scopes_present: bool,
    pub maybe_stale: Option<MaybeStaleAdmission>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaybeStaleAdmission {
    pub unchanged_at_admission: bool,
}
