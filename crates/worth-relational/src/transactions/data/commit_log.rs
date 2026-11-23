use crate::diagnostics::data::DiagnosticCode;
use crate::history::data::coordinates::CommitId;
use crate::publication::bundle::publication_status::PublicationStage;
use crate::publication::patch::data::PatchStreamPosition;
use crate::snapshots::data::coordinates::SnapshotId;
use crate::validation::data::execution::InvariantExecutionPoint;
use crate::validation::data::execution_posture::InvariantExecutionDisposition;
use crate::validation::data::groups::InvariantGroupSet;
use crate::validation::data::observation_kind::InvariantObservationKind;
use serde::{Deserialize, Serialize};

use crate::transactions::data::preparation_strategy_posture::{
    ParallelLegality, ParallelProfitability, PreparationStrategySelection, SerialPreparationReason,
};
use worth_foundational::facade::AspectKey;

use crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary;

/// Descriptive evidence of an invariant pass that can be recorded in a commit log.
pub struct CommitInvariantEvaluation {
    pub(crate) execution_point: InvariantExecutionPoint,
    pub(crate) observation_kind: InvariantObservationKind,
    pub(crate) disposition: InvariantExecutionDisposition,
    pub(crate) preparation_selected_mode: Option<PreparationStrategySelection>,
    pub(crate) preparation_parallel_legality: Option<ParallelLegality>,
    pub(crate) preparation_parallel_profitability: Option<ParallelProfitability>,
    pub(crate) preparation_serial_selection_reason: Option<SerialPreparationReason>,
    pub(crate) consumed_groups: InvariantGroupSet,
    pub(crate) applicable_groups: InvariantGroupSet,
    pub(crate) result_count: usize,
    pub(crate) advisory_count: usize,
    pub(crate) violation_count: usize,
    pub(crate) blocking_violation: bool,
    pub(crate) publication_violation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitPhase {
    DraftPreparation,
    InvariantPreCheck,
    AuthoritativeMutation,
    HistoryResolution,
    InvariantPostCheck,
    ArtifactAssembly,
    DurableAppend,
    Publication,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitHistorySummary {
    pub target_branch: String,
    pub requested_merge_parent_count: usize,
    pub effective_merge_parent_count: usize,
    pub parent_count: usize,
    pub merge_base_count: usize,
    pub had_previous_branch_head: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitPatchBudgetSummary {
    pub patch_record_count: usize,
    pub max_patch_records_per_commit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitChangeSummary {
    pub changed_record_count: usize,
    pub adjacency_delta_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAspectSummary {
    pub changed_entity_aspect_count: usize,
    pub changed_relation_aspect_count: usize,
    pub touched_aspects: Vec<AspectKey>,
    pub opaque_aspect_delta_count: usize,
    pub zero_aspect_structural_delta_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitPublicationSummary {
    pub patch_record_count: usize,
    pub diagnostics_entry_count: usize,
    pub lineage_event_count: usize,
    pub patch_position: Option<PatchStreamPosition>,
    pub final_snapshot_id: Option<SnapshotId>,
    pub merge_parent_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitTraceEvent {
    PhaseStarted(CommitPhase),
    PhaseCompleted(CommitPhase),
    StructuralSummary,
    ChangedRecordsPrepared,
    InvariantEvaluated {
        execution_point: InvariantExecutionPoint,
        observation_kind: InvariantObservationKind,
        disposition: InvariantExecutionDisposition,
        preparation_selected_mode: Option<PreparationStrategySelection>,
        preparation_parallel_legality: Option<ParallelLegality>,
        preparation_parallel_profitability: Option<ParallelProfitability>,
        preparation_serial_selection_reason: Option<SerialPreparationReason>,
        consumed_groups: InvariantGroupSet,
        applicable_groups: InvariantGroupSet,
        result_count: usize,
        advisory_count: usize,
        violation_count: usize,
        blocking_violation: bool,
        publication_violation: bool,
    },
    HistoryResolved,
    PublicationArtifactsPrepared,
    PatchBudgetEvaluated,
    AspectSummaryPrepared,
    DurableAppendPrepared {
        commit_id: CommitId,
        branch_id: String,
        patch_position: PatchStreamPosition,
    },
    CommitPublished {
        commit_id: CommitId,
        branch_id: String,
    },
    CommitRejected {
        phase: CommitPhase,
        diagnostic_code: Option<DiagnosticCode>,
        publication_stage: Option<PublicationStage>,
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CommitSummary {
    pub phase_count: usize,
    pub structural_summary: Option<CommitStructuralSummary>,
    pub history_summary: Option<CommitHistorySummary>,
    pub patch_budget_summary: Option<CommitPatchBudgetSummary>,
    pub change_summary: Option<CommitChangeSummary>,
    pub aspect_summary: Option<CommitAspectSummary>,
    pub publication_summary: Option<CommitPublicationSummary>,
    pub invariant_result_count: usize,
    pub invariant_advisory_count: usize,
    pub invariant_violation_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CommitLog {
    events: Vec<CommitTraceEvent>,
    running_summary: CommitSummary,
}

impl CommitLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[CommitTraceEvent] {
        &self.events
    }

    pub fn summary(&self) -> &CommitSummary {
        &self.running_summary
    }

    pub fn has_phase_started(&self, phase: CommitPhase) -> bool {
        self.events.iter().any(
            |event| matches!(event, CommitTraceEvent::PhaseStarted(started) if *started == phase),
        )
    }

    pub fn has_phase_completed(&self, phase: CommitPhase) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::PhaseCompleted(completed) if *completed == phase))
    }

    pub fn structural_summary_event(&self) -> Option<&CommitStructuralSummary> {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::StructuralSummary))
            .then_some(self.running_summary.structural_summary.as_ref())
            .flatten()
    }

    pub fn history_summary_event(&self) -> Option<&CommitHistorySummary> {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::HistoryResolved))
            .then_some(self.running_summary.history_summary.as_ref())
            .flatten()
    }

    pub fn change_summary_event(&self) -> Option<&CommitChangeSummary> {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::ChangedRecordsPrepared))
            .then_some(self.running_summary.change_summary.as_ref())
            .flatten()
    }

    pub fn patch_budget_summary_event(&self) -> Option<&CommitPatchBudgetSummary> {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::PatchBudgetEvaluated))
            .then_some(self.running_summary.patch_budget_summary.as_ref())
            .flatten()
    }

    pub fn aspect_summary_event(&self) -> Option<&CommitAspectSummary> {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::AspectSummaryPrepared))
            .then_some(self.running_summary.aspect_summary.as_ref())
            .flatten()
    }

    pub fn publication_summary_event(&self) -> Option<&CommitPublicationSummary> {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::PublicationArtifactsPrepared))
            .then_some(self.running_summary.publication_summary.as_ref())
            .flatten()
    }

    pub fn has_commit_published(&self) -> bool {
        self.events
            .iter()
            .any(|event| matches!(event, CommitTraceEvent::CommitPublished { .. }))
    }

    pub fn has_rejection(
        &self,
        phase: CommitPhase,
        diagnostic_code: Option<DiagnosticCode>,
        publication_stage: Option<PublicationStage>,
    ) -> bool {
        self.events.iter().any(|event| {
            matches!(
                event,
                CommitTraceEvent::CommitRejected {
                    phase: rejected_phase,
                    diagnostic_code: rejected_code,
                    publication_stage: rejected_stage,
                    ..
                } if *rejected_phase == phase
                    && *rejected_code == diagnostic_code
                    && *rejected_stage == publication_stage
            )
        })
    }

    pub fn has_rejection_code(&self, diagnostic_code: DiagnosticCode) -> bool {
        self.events.iter().any(|event| {
            matches!(
                event,
                CommitTraceEvent::CommitRejected {
                    diagnostic_code: Some(rejected_code),
                    ..
                } if *rejected_code == diagnostic_code
            )
        })
    }

    pub fn begin_phase(&mut self, phase: CommitPhase) {
        self.events.push(CommitTraceEvent::PhaseStarted(phase));
        self.running_summary.phase_count += 1;
    }

    pub fn complete_phase(&mut self, phase: CommitPhase) {
        self.events.push(CommitTraceEvent::PhaseCompleted(phase));
    }

    pub fn record_structural_summary(&mut self, summary: &CommitStructuralSummary) {
        self.running_summary.structural_summary = Some(summary.clone());
        self.events.push(CommitTraceEvent::StructuralSummary);
    }

    pub fn record_changed_records(&mut self, summary: &CommitChangeSummary) {
        self.running_summary.change_summary = Some(summary.clone());
        self.events.push(CommitTraceEvent::ChangedRecordsPrepared);
    }

    pub fn record_invariant_outcomes(&mut self, outcome: impl Into<CommitInvariantEvaluation>) {
        let outcome = outcome.into();
        self.running_summary.invariant_result_count += outcome.result_count;
        self.running_summary.invariant_advisory_count += outcome.advisory_count;
        self.running_summary.invariant_violation_count += outcome.violation_count;
        self.events.push(CommitTraceEvent::InvariantEvaluated {
            execution_point: outcome.execution_point,
            observation_kind: outcome.observation_kind,
            disposition: outcome.disposition,
            preparation_selected_mode: outcome.preparation_selected_mode,
            preparation_parallel_legality: outcome.preparation_parallel_legality,
            preparation_parallel_profitability: outcome.preparation_parallel_profitability,
            preparation_serial_selection_reason: outcome.preparation_serial_selection_reason,
            consumed_groups: outcome.consumed_groups,
            applicable_groups: outcome.applicable_groups,
            result_count: outcome.result_count,
            advisory_count: outcome.advisory_count,
            violation_count: outcome.violation_count,
            blocking_violation: outcome.blocking_violation,
            publication_violation: outcome.publication_violation,
        });
    }

    pub fn record_history_resolution(&mut self, summary: &CommitHistorySummary) {
        self.running_summary.history_summary = Some(summary.clone());
        self.events.push(CommitTraceEvent::HistoryResolved);
    }

    pub fn record_publication_artifacts(&mut self, summary: &CommitPublicationSummary) {
        self.running_summary.publication_summary = Some(summary.clone());
        self.events
            .push(CommitTraceEvent::PublicationArtifactsPrepared);
    }

    pub(crate) fn record_publication_position(&mut self, position: PatchStreamPosition) {
        if let Some(summary) = self.running_summary.publication_summary.as_mut() {
            summary.patch_position = Some(position);
        }
    }

    pub fn record_patch_budget(&mut self, summary: &CommitPatchBudgetSummary) {
        self.running_summary.patch_budget_summary = Some(summary.clone());
        self.events.push(CommitTraceEvent::PatchBudgetEvaluated);
    }

    pub fn record_aspect_summary(&mut self, summary: &CommitAspectSummary) {
        self.running_summary.aspect_summary = Some(summary.clone());
        self.events.push(CommitTraceEvent::AspectSummaryPrepared);
    }

    pub fn record_durable_append_prepared(
        &mut self,
        commit_id: CommitId,
        branch_id: &str,
        patch_position: PatchStreamPosition,
    ) {
        self.events.push(CommitTraceEvent::DurableAppendPrepared {
            commit_id,
            branch_id: branch_id.to_string(),
            patch_position,
        });
    }

    pub fn record_commit_published(&mut self, commit_id: CommitId, branch_id: &str) {
        self.events.push(CommitTraceEvent::CommitPublished {
            commit_id,
            branch_id: branch_id.to_string(),
        });
    }

    pub fn record_rejection(
        &mut self,
        phase: CommitPhase,
        diagnostic_code: Option<DiagnosticCode>,
        publication_stage: Option<PublicationStage>,
        detail: impl Into<String>,
    ) {
        self.events.push(CommitTraceEvent::CommitRejected {
            phase,
            diagnostic_code,
            publication_stage,
            detail: detail.into(),
        });
    }
}
