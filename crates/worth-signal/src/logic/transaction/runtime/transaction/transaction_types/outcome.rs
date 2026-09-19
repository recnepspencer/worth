use serde::{Deserialize, Serialize};

use crate::data::telemetry::RuntimeTelemetry;
use crate::data::temporal::TemporalExecutionSummary;
use crate::diagnostics::epochs::EventEpochSummary;
use crate::diagnostics::replay::ReplayEventKind;
use crate::logic::planner::{ExecutionRecordId, ExecutionReport, SemanticSegmentId};

use super::super::super::state::ReconstructabilityRecord;
use super::super::transaction_observation::ObservationBoundarySummary;

use super::evidence::TemporalTransactionEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionOutcome {
    Committed,
    RolledBack,
    Poisoned,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TransactionTiming {
    pub total_nanos: u128,
    pub evaluation_nanos: u128,
    pub event_flush_nanos: u128,
    pub commit_nanos: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EvaluationSummary {
    pub nodes_evaluated: u32,
    pub nodes_recomputed: u32,
    pub nodes_suppressed: u32,
    pub plans_built: u32,
    pub stages_executed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionResult {
    pub(crate) diagnostic_publication_work: crate::diagnostics::state::DiagnosticPublicationWork,
    pub outcome: TransactionOutcome,
    pub execution_report: Option<ExecutionReport>,
    pub timing: TransactionTiming,
    pub touched_nodes: u32,
    pub evaluation_summary: EvaluationSummary,
    pub temporal_summary: TemporalExecutionSummary,
    pub temporal_evidence: TemporalTransactionEvidence,
    pub reconstructability: ReconstructabilityRecord,
    pub event_epochs: Vec<EventEpochSummary>,
    pub rollback: Option<crate::diagnostics::failure::RollbackDiagnostic>,
    pub warnings: Vec<super::super::envelope::AdvisoryRecord>,
    pub observation: ObservationBoundarySummary,
    pub decision_summary: super::super::envelope::DecisionSummary,
    pub decision_log: super::super::envelope::DecisionLog,
    pub integrity_markers: super::super::envelope::IntegrityMarkers,
    pub performance_accounting: RuntimeTelemetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionReplayEntry {
    pub kind: ReplayEventKind,
    pub detail: String,
    pub execution_record_id: Option<ExecutionRecordId>,
    pub semantic_segment_id: Option<SemanticSegmentId>,
}
