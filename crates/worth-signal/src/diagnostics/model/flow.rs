mod retained_charge;

use serde::{Deserialize, Serialize};

mod retained_view;
pub use retained_view::RetainedFlowSummaryView;

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::proof::{DedupedNodeBatch, FrontierDiagnosticsSidecar};
use crate::diagnostics::epochs::EventEpochSummary;
use crate::diagnostics::failure::RollbackDiagnostic;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::{
    EvaluationPlanSummary, ExecutionReportSummary, ExplanationSummary,
};
use crate::logic::planner::{EvaluationPlan, ExecutionReport, StageExecutor};
use crate::logic::transaction::ObservationBoundarySummary;

/// Structured summary of one upstream change input to signal execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeInputSummary {
    pub changed_nodes: Vec<NodeId>,
    pub changed_aspects: Vec<u8>,
    pub changed_region_count: u32,
    pub causality_kind: Option<String>,
}

/// Structured summary of invalidation routing for one flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationSummary {
    pub invalidated_direct_subscribers: u32,
    pub maybe_stale_direct_subscribers: u32,
    pub partition_scoped_checks: u32,
    pub narrowed_frontier_width: u32,
    pub transitive_frontier_width: u32,
    #[serde(default)]
    pub frontier_seed_count: u32,
    #[serde(default)]
    pub frontier_group_count: u32,
    #[serde(default)]
    pub frontier_direct_wave_count: u32,
    #[serde(default)]
    pub frontier_transitive_wave_count: u32,
    #[serde(default)]
    pub frontier_partition_match_count: u32,
    #[serde(default)]
    pub frontier_detail_match_count: u32,
    #[serde(default)]
    pub frontier_cycle_check_candidate_count: u32,
    #[serde(default)]
    pub frontier_cycle_check_visited_count: u32,
    #[serde(default)]
    pub frontier_trace_retained_count: u32,
}

/// End-to-end causal summary for one signal execution flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowSummary {
    pub profile: DiagnosticsTier,
    pub change: ChangeInputSummary,
    pub invalidation: InvalidationSummary,
    pub planning: PlanningSummary,
    pub precompute: PrecomputeSummary,
    pub apply: ApplySummary,
    #[serde(default)]
    pub cause_samples: Vec<FlowCauseSample>,
    #[serde(default)]
    pub event_epochs: Vec<EventEpochSummary>,
    #[serde(default)]
    pub observation: Option<ObservationBoundarySummary>,
    pub rollback: Option<RollbackSummary>,
    pub explanation: Option<ExplanationSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowCauseSample {
    pub node: NodeId,
    pub cause_kinds: Vec<String>,
    pub scope_kinds: Vec<String>,
    pub scope_notes: Vec<String>,
    pub suspect_classes: Vec<String>,
    pub rewired: bool,
    pub conservative_recompute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningSummary {
    pub plan: EvaluationPlanSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrecomputeSummary {
    pub executor: Option<StageExecutor>,
    pub stage_count: u32,
    pub task_count: u32,
    pub prepared_evaluations_produced: u32,
    pub tasks_deferred_by_condition: u32,
    pub tasks_satisfied_by_memoization: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplySummary {
    pub report: ExecutionReportSummary,
    pub prepared_evaluations_applied: u32,
    pub dependency_capture_updates: u32,
    pub tasks_validated_clean: u32,
    pub tasks_pruned: u32,
    pub tasks_with_suppressed_propagation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackSummary {
    pub rolled_back: bool,
    pub staged_node_patch_count: u64,
    pub max_touched_nodes_in_txn: u64,
    pub reason: Option<String>,
}

impl ChangeInputSummary {
    pub fn new(
        changed_nodes: Vec<NodeId>,
        changed_aspects: Vec<Aspect>,
        changed_region_count: u32,
        causality_kind: Option<String>,
    ) -> Self {
        let changed_nodes = DedupedNodeBatch::canonicalize_unordered(changed_nodes).into_vec();
        let changed_aspects = canonicalize_changed_aspects(changed_aspects)
            .into_iter()
            .map(|aspect| aspect.index() as u8)
            .collect();
        Self {
            changed_nodes,
            changed_aspects,
            changed_region_count,
            causality_kind,
        }
    }
}

impl InvalidationSummary {
    pub fn empty_frontier() -> Self {
        Self::from_frontier_execution(&FrontierDiagnosticsSidecar::new(
            0,
            Vec::new(),
            Vec::new(),
            Default::default(),
            Default::default(),
        ))
    }

    pub(crate) fn from_frontier_execution(summary: &FrontierDiagnosticsSidecar) -> Self {
        let invalidated_direct_subscribers = summary
            .direct_waves
            .iter()
            .flat_map(|wave| wave.entries.iter())
            .filter(|entry| {
                matches!(
                    entry.classification,
                    crate::data::proof::FrontierEntryClassification::DirectDirty
                )
            })
            .count() as u32;
        let maybe_stale_direct_subscribers = summary
            .direct_waves
            .iter()
            .flat_map(|wave| wave.entries.iter())
            .filter(|entry| {
                matches!(
                    entry.classification,
                    crate::data::proof::FrontierEntryClassification::MaybeStale
                )
            })
            .count() as u32;
        let narrowed_frontier_width = summary
            .direct_waves
            .iter()
            .map(|wave| wave.entries.len())
            .sum::<usize>() as u32;
        let transitive_frontier_width = summary
            .transitive_waves
            .iter()
            .map(|wave| wave.entries.len())
            .sum::<usize>() as u32;

        Self::new(
            invalidated_direct_subscribers,
            maybe_stale_direct_subscribers,
            summary.counters.frontier_partition_scoped_check_count as u32,
            narrowed_frontier_width,
            transitive_frontier_width,
        )
        .with_frontier_counters(
            summary.counters.frontier_seed_count as u32,
            summary.counters.frontier_group_count as u32,
            summary.counters.frontier_direct_wave_count as u32,
            summary.counters.frontier_transitive_wave_count as u32,
            summary.counters.frontier_partition_match_count as u32,
            summary.counters.frontier_detail_match_count as u32,
            summary.counters.frontier_cycle_check_candidate_count as u32,
            summary.counters.frontier_cycle_check_visited_count as u32,
            summary.counters.frontier_trace_retained_count as u32,
        )
    }

    pub fn new(
        invalidated_direct_subscribers: u32,
        maybe_stale_direct_subscribers: u32,
        partition_scoped_checks: u32,
        narrowed_frontier_width: u32,
        transitive_frontier_width: u32,
    ) -> Self {
        Self {
            invalidated_direct_subscribers,
            maybe_stale_direct_subscribers,
            partition_scoped_checks,
            narrowed_frontier_width,
            transitive_frontier_width,
            frontier_seed_count: 0,
            frontier_group_count: 0,
            frontier_direct_wave_count: 0,
            frontier_transitive_wave_count: 0,
            frontier_partition_match_count: 0,
            frontier_detail_match_count: 0,
            frontier_cycle_check_candidate_count: 0,
            frontier_cycle_check_visited_count: 0,
            frontier_trace_retained_count: 0,
        }
    }

    pub fn with_frontier_counters(
        mut self,
        frontier_seed_count: u32,
        frontier_group_count: u32,
        frontier_direct_wave_count: u32,
        frontier_transitive_wave_count: u32,
        frontier_partition_match_count: u32,
        frontier_detail_match_count: u32,
        frontier_cycle_check_candidate_count: u32,
        frontier_cycle_check_visited_count: u32,
        frontier_trace_retained_count: u32,
    ) -> Self {
        self.frontier_seed_count = frontier_seed_count;
        self.frontier_group_count = frontier_group_count;
        self.frontier_direct_wave_count = frontier_direct_wave_count;
        self.frontier_transitive_wave_count = frontier_transitive_wave_count;
        self.frontier_partition_match_count = frontier_partition_match_count;
        self.frontier_detail_match_count = frontier_detail_match_count;
        self.frontier_cycle_check_candidate_count = frontier_cycle_check_candidate_count;
        self.frontier_cycle_check_visited_count = frontier_cycle_check_visited_count;
        self.frontier_trace_retained_count = frontier_trace_retained_count;
        self
    }
}

impl FlowSummary {
    pub fn new(
        profile: DiagnosticsTier,
        change: ChangeInputSummary,
        invalidation: InvalidationSummary,
        planning: PlanningSummary,
        precompute: PrecomputeSummary,
        apply: ApplySummary,
        cause_samples: Vec<FlowCauseSample>,
        event_epochs: Vec<EventEpochSummary>,
        observation: Option<ObservationBoundarySummary>,
        rollback: Option<RollbackSummary>,
        explanation: Option<ExplanationSummary>,
    ) -> Self {
        Self {
            profile,
            change,
            invalidation,
            planning,
            precompute,
            apply,
            cause_samples,
            event_epochs,
            observation,
            rollback,
            explanation,
        }
    }
}

impl PlanningSummary {
    pub fn from_plan(plan: &EvaluationPlan, profile: DiagnosticsTier) -> Self {
        Self {
            plan: EvaluationPlanSummary::from_plan(plan, profile),
        }
    }

    pub fn from_summary(plan: EvaluationPlanSummary) -> Self {
        Self { plan }
    }
}

impl PrecomputeSummary {
    pub fn from_report(report: &ExecutionReport, _profile: DiagnosticsTier) -> Self {
        let executor = report.stages.first().map(|stage| match stage.outcome {
            crate::logic::planner::StageExecutionOutcome::CompletedSerial => StageExecutor::Serial,
            #[cfg(feature = "parallel")]
            crate::logic::planner::StageExecutionOutcome::CompletedParallel => {
                StageExecutor::parallel(1)
            }
        });
        Self {
            executor,
            stage_count: report.stage_count,
            task_count: report.task_count,
            prepared_evaluations_produced: report.prepared_evaluations_produced,
            tasks_deferred_by_condition: report.tasks_deferred_by_condition,
            tasks_satisfied_by_memoization: report.tasks_satisfied_by_memoization,
        }
    }
}

impl ApplySummary {
    pub fn from_report(report: &ExecutionReport, profile: DiagnosticsTier) -> Self {
        Self {
            report: ExecutionReportSummary::from_report(report, profile),
            prepared_evaluations_applied: report.prepared_evaluations_applied,
            dependency_capture_updates: report.dependency_capture_updates,
            tasks_validated_clean: report.tasks_validated_clean,
            tasks_pruned: report.tasks_pruned,
            tasks_with_suppressed_propagation: report.tasks_with_suppressed_propagation,
        }
    }
}

impl RollbackSummary {
    pub fn from_rollback(rollback: &RollbackDiagnostic) -> Self {
        Self {
            rolled_back: rollback.rolled_back,
            staged_node_patch_count: rollback.staged_node_patch_count,
            max_touched_nodes_in_txn: rollback.max_touched_nodes_in_txn,
            reason: rollback.reason.clone(),
        }
    }
}

fn canonicalize_changed_aspects(mut changed_aspects: Vec<Aspect>) -> Vec<Aspect> {
    if changed_aspects.len() > 1 {
        changed_aspects.sort_by_key(|aspect| aspect.index());
        changed_aspects.dedup();
    }
    changed_aspects
}
