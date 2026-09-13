mod branch_carrier_charge;
mod branching;
mod fork;
mod fork_growth;
mod retained_charge;
pub(crate) use branch_carrier_charge::BranchCarrierChargeDenial;
mod history;
mod indexes;
mod issuance;
pub(crate) use history::{DiagnosticHistory, DiagnosticHistoryEditDenial};
mod lifecycle;
mod lineage;
mod lineage_publication;
pub(crate) use lineage_publication::LineagePublicationDenial;
mod replay;
mod restore_history;
mod retained;
mod retained_flow;
mod snapshot;
#[cfg(test)]
mod summary_sharing_tests;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::data::persistent_ord_set::PersistentOrdSet;
use crate::data::proof::{
    FrontierDiagnosticsSidecar, InvalidationPlanningEstimate, InvalidationTraceRecord,
};
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::failure::{FailureSummary, RollbackDiagnostic};
use crate::diagnostics::lineage::{LineageArtifactId, LineageRecord};
use crate::diagnostics::replay::{ReplayCursor, ReplayEvent};
use crate::diagnostics::summary::{ExecutionHistorySummary, GraphSummary};
use crate::logic::transaction::ObservationBoundarySummary;
use crate::runtime_policy::SignalRuntimePolicy;
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};
use retained_flow::RetainedFlow;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DiagnosticsState {
    #[serde(default)]
    request_mirror: SignalRuntimePolicy,
    #[serde(skip)]
    installed_retention_budget: crate::diagnostics::policy::RetentionBudget,
    #[serde(skip)]
    installed_tier: crate::diagnostics::profile::DiagnosticsTier,
    #[serde(skip)]
    installed_frontier_tracing_policy: crate::diagnostics::policy::FrontierTracingPolicy,
    #[serde(default)]
    latest_flow: Option<RetainedFlow>,
    #[serde(default)]
    latest_failure: Option<Arc<FailureSummary>>,
    #[serde(default)]
    latest_rollback: Option<Arc<RollbackDiagnostic>>,
    #[serde(default)]
    latest_observation: Option<Arc<ObservationBoundarySummary>>,
    #[serde(default)]
    latest_graph_summary: Option<Arc<GraphSummary>>,
    #[serde(default)]
    pending_graph_summary: Option<Arc<GraphSummary>>,
    #[serde(default)]
    recent_history: DiagnosticHistory<ExecutionHistorySummary>,
    #[serde(default)]
    replay_events: DiagnosticHistory<ReplayEvent>,
    #[serde(default)]
    lineage_records: DiagnosticHistory<LineageRecord>,
    #[serde(skip)]
    replay_events_by_branch: PersistentOrdMap<SignalBranchId, DiagnosticHistory<ReplayEvent>>,
    #[serde(skip)]
    replay_events_by_node: PersistentOrdMap<NodeId, DiagnosticHistory<ReplayEvent>>,
    #[serde(skip)]
    replay_events_by_artifact: PersistentOrdMap<LineageArtifactId, DiagnosticHistory<ReplayEvent>>,
    #[serde(skip)]
    replay_cursor_offsets: PersistentOrdMap<ReplayCursor, usize>,
    #[serde(skip, default)]
    replay_cursor_offset_base: usize,
    #[serde(skip)]
    snapshot_replay_cursors: PersistentOrdMap<SignalSnapshotId, ReplayCursor>,
    #[serde(skip)]
    lineage_records_by_artifact:
        PersistentOrdMap<LineageArtifactId, DiagnosticHistory<LineageRecord>>,
    #[serde(skip)]
    lineage_records_by_node: PersistentOrdMap<NodeId, DiagnosticHistory<LineageRecord>>,
    #[serde(default)]
    explanation_facts: PersistentOrdMap<NodeId, ExplanationFact>,
    #[serde(default)]
    provenance_facts: PersistentOrdMap<NodeId, ProvenanceFact>,
    #[serde(default)]
    branch_catalog: PersistentOrdMap<SignalBranchId, SignalBranchHandle>,
    #[serde(default)]
    active_branch: SignalBranchId,
    #[serde(default)]
    next_replay_cursor: u64,
    #[serde(default)]
    next_snapshot_id: u64,
    #[serde(default)]
    next_branch_id: u64,
    #[serde(default)]
    next_lineage_artifact_id: u64,
    #[serde(default)]
    next_lineage_sequence: u64,
    #[serde(default)]
    pending_input: Option<PendingFlowInput>,
    #[serde(default)]
    latest_frontier_execution: Option<Arc<FrontierDiagnosticsSidecar>>,
    #[serde(default)]
    latest_invalidation_planning_estimate: Option<InvalidationPlanningEstimate>,
    #[serde(default)]
    latest_invalidation_trace_records: Arc<Vec<InvalidationTraceRecord>>,
    /// Surfaces that have been explicitly activated for observation on this
    /// graph. This remains separate from the current policy so historical
    /// reads can distinguish inactive evidence from policy omission.
    #[serde(default)]
    observation_activation_mask: u8,
    #[serde(skip)]
    lineage_custody: lineage_publication::LineageRetentionCustody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PendingFlowInput {
    changed_nodes: PersistentOrdSet<NodeId>,
    changed_aspects: PersistentOrdSet<u8>,
    changed_region_count: u32,
    causality_kind: Option<Arc<String>>,
}
