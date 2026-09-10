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
