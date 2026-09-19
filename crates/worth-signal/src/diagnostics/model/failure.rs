mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::diagnostics::epochs::EventEpochSummary;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::logic::planner::{ExecutionRecordId, PlanSummary, StageExecutor};

/// High-level phase where an execution failure occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionFailurePhase {
    Invalidation,
    Planning,
    SnapshotBuild,
    Precompute,
    Apply,
    Rollback,
    CommitPromotion,
    ParallelDivergence,
}

/// Structured context for one execution failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFailureContext {
    pub phase: ExecutionFailurePhase,
    pub stage_index: Option<u32>,
    pub node: Option<NodeId>,
    pub executor: Option<StageExecutor>,
    pub execution_record_id: Option<ExecutionRecordId>,
    pub plan_summary: Option<PlanSummary>,
    pub message: String,
}

/// Structured rollback diagnostic for staged execution failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackDiagnostic {
    pub rolled_back: bool,
    pub staged_node_patch_count: u64,
    pub max_touched_nodes_in_txn: u64,
    pub reason: Option<String>,
    #[serde(default)]
    pub event_epochs: Vec<EventEpochSummary>,
}

/// Compact failure summary suitable for comparison and persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureSummary {
    pub profile: DiagnosticsTier,
    pub phase: ExecutionFailurePhase,
    pub stage_index: Option<u32>,
    pub node: Option<NodeId>,
    pub executor: Option<StageExecutor>,
    pub execution_record_id: Option<ExecutionRecordId>,
    pub has_plan_summary: bool,
    pub rolled_back: bool,
    pub staged_node_patch_count: Option<u64>,
    pub max_touched_nodes_in_txn: Option<u64>,
    #[serde(default)]
    pub event_epochs: Vec<EventEpochSummary>,
    pub message: String,
}

impl FailureSummary {
    pub(crate) fn suppressed(profile: DiagnosticsTier, phase: ExecutionFailurePhase) -> Self {
        Self {
            profile,
            phase,
            stage_index: None,
            node: None,
            executor: None,
            execution_record_id: None,
            has_plan_summary: false,
            rolled_back: false,
            staged_node_patch_count: None,
            max_touched_nodes_in_txn: None,
            event_epochs: Vec::new(),
            message: String::new(),
        }
    }
}

impl ExecutionFailureContext {
    pub fn new(
        phase: ExecutionFailurePhase,
        stage_index: Option<u32>,
        node: Option<NodeId>,
        executor: Option<StageExecutor>,
        execution_record_id: Option<ExecutionRecordId>,
        plan_summary: Option<PlanSummary>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            stage_index,
            node,
            executor,
            execution_record_id,
            plan_summary,
            message: message.into(),
        }
    }

    pub fn from_error(
        phase: ExecutionFailurePhase,
        error: &SignalError,
        plan_summary: Option<PlanSummary>,
    ) -> Self {
        Self::new(
            phase,
            None,
            None,
            None,
            None,
            plan_summary,
            error.to_string(),
        )
    }

    pub fn summarize(
        &self,
        rollback: Option<&RollbackDiagnostic>,
        profile: DiagnosticsTier,
    ) -> FailureSummary {
        FailureSummary {
            profile,
            phase: self.phase,
            stage_index: self.stage_index,
            node: self.node,
            executor: self.executor,
            execution_record_id: self.execution_record_id,
            has_plan_summary: self.plan_summary.is_some(),
            rolled_back: rollback.map(|d| d.rolled_back).unwrap_or(false),
            staged_node_patch_count: rollback.map(|d| d.staged_node_patch_count),
            max_touched_nodes_in_txn: rollback.map(|d| d.max_touched_nodes_in_txn),
            event_epochs: rollback
                .map(|diagnostic| diagnostic.event_epochs.clone())
                .unwrap_or_default(),
            message: self.message.clone(),
        }
    }
}

impl RollbackDiagnostic {
    pub fn new(
        rolled_back: bool,
        staged_node_patch_count: u64,
        max_touched_nodes_in_txn: u64,
        reason: Option<String>,
        event_epochs: Vec<EventEpochSummary>,
    ) -> Self {
        Self {
            rolled_back,
            staged_node_patch_count,
            max_touched_nodes_in_txn,
            reason,
            event_epochs,
        }
    }
}
