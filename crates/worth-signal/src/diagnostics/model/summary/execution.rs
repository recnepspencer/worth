mod retained_charge;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::diagnostics::profile::DiagnosticsTier;
use crate::logic::planner::{ExecutionReport, StageExecutionOutcome, TaskExecutionOutcome};

pub type TaskOutcomeCounts = BTreeMap<TaskExecutionOutcome, u32>;
pub type StageOutcomeCounts = BTreeMap<StageExecutionOutcome, u32>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReportSummary {
    pub profile: DiagnosticsTier,
    pub stage_count: u32,
    pub task_count: u32,
    pub tasks_executed: u32,
    pub tasks_pruned: u32,
    pub tasks_validated_clean: u32,
    pub tasks_deferred_by_condition: u32,
    pub tasks_reverted_clean_by_condition: u32,
    pub tasks_satisfied_by_memoization: u32,
    pub tasks_with_suppressed_propagation: u32,
    pub prepared_evaluations_produced: u32,
    pub prepared_evaluations_applied: u32,
    pub dependency_capture_updates: u32,
    pub semantic_segment_count: u32,
    pub temporal_summary: crate::data::temporal::TemporalExecutionSummary,
    pub task_outcome_counts: TaskOutcomeCounts,
    pub stage_outcome_counts: StageOutcomeCounts,
}

impl ExecutionReportSummary {
    /// Adds a later execution report of the same flow.
    pub fn absorb(&mut self, other: ExecutionReportSummary) {
        self.stage_count = self.stage_count.saturating_add(other.stage_count);
        self.task_count = self.task_count.saturating_add(other.task_count);
        self.tasks_executed = self.tasks_executed.saturating_add(other.tasks_executed);
        self.tasks_pruned = self.tasks_pruned.saturating_add(other.tasks_pruned);
        self.tasks_validated_clean = self
            .tasks_validated_clean
            .saturating_add(other.tasks_validated_clean);
        self.tasks_deferred_by_condition = self
            .tasks_deferred_by_condition
            .saturating_add(other.tasks_deferred_by_condition);
        self.tasks_reverted_clean_by_condition = self
            .tasks_reverted_clean_by_condition
            .saturating_add(other.tasks_reverted_clean_by_condition);
        self.tasks_satisfied_by_memoization = self
            .tasks_satisfied_by_memoization
            .saturating_add(other.tasks_satisfied_by_memoization);
        self.tasks_with_suppressed_propagation = self
            .tasks_with_suppressed_propagation
            .saturating_add(other.tasks_with_suppressed_propagation);
        self.prepared_evaluations_produced = self
            .prepared_evaluations_produced
            .saturating_add(other.prepared_evaluations_produced);
        self.prepared_evaluations_applied = self
            .prepared_evaluations_applied
            .saturating_add(other.prepared_evaluations_applied);
        self.dependency_capture_updates = self
            .dependency_capture_updates
            .saturating_add(other.dependency_capture_updates);
        self.semantic_segment_count = self
            .semantic_segment_count
            .saturating_add(other.semantic_segment_count);
        self.temporal_summary.absorb(other.temporal_summary);
        for (outcome, count) in other.task_outcome_counts {
            let entry = self.task_outcome_counts.entry(outcome).or_insert(0);
            *entry = entry.saturating_add(count);
        }
        for (outcome, count) in other.stage_outcome_counts {
            let entry = self.stage_outcome_counts.entry(outcome).or_insert(0);
            *entry = entry.saturating_add(count);
        }
    }

    pub fn from_report(report: &ExecutionReport, profile: DiagnosticsTier) -> Self {
        let mut task_outcome_counts = TaskOutcomeCounts::new();
        let mut stage_outcome_counts = StageOutcomeCounts::new();
        for stage in &report.stages {
            *stage_outcome_counts.entry(stage.outcome).or_insert(0) += 1;
            for task in &stage.task_records {
                *task_outcome_counts.entry(task.outcome).or_insert(0) += 1;
            }
        }

        Self {
            profile,
            stage_count: report.stage_count,
            task_count: report.task_count,
            tasks_executed: report.tasks_executed,
            tasks_pruned: report.tasks_pruned,
            tasks_validated_clean: report.tasks_validated_clean,
            tasks_deferred_by_condition: report.tasks_deferred_by_condition,
            tasks_reverted_clean_by_condition: report.tasks_reverted_clean_by_condition,
            tasks_satisfied_by_memoization: report.tasks_satisfied_by_memoization,
            tasks_with_suppressed_propagation: report.tasks_with_suppressed_propagation,
            prepared_evaluations_produced: report.prepared_evaluations_produced,
            prepared_evaluations_applied: report.prepared_evaluations_applied,
            dependency_capture_updates: report.dependency_capture_updates,
            semantic_segment_count: report.semantic_segment_count,
            temporal_summary: report.temporal_summary,
            task_outcome_counts,
            stage_outcome_counts,
        }
    }
}

impl ExecutionReport {
    pub fn diagnostics_summary(&self, profile: DiagnosticsTier) -> ExecutionReportSummary {
        ExecutionReportSummary::from_report(self, profile)
    }
}
