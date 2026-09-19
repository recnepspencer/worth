use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::ExecutionReportSummary;
use crate::logic::planner::ExecutionReport;

use super::ApplySummary;

impl ApplySummary {
    /// Adds the apply work of a later execution of the same flow.
    pub fn absorb(&mut self, other: ApplySummary) {
        self.report.absorb(other.report);
        self.prepared_evaluations_applied = self
            .prepared_evaluations_applied
            .saturating_add(other.prepared_evaluations_applied);
        self.dependency_capture_updates = self
            .dependency_capture_updates
            .saturating_add(other.dependency_capture_updates);
        self.tasks_validated_clean = self
            .tasks_validated_clean
            .saturating_add(other.tasks_validated_clean);
        self.tasks_pruned = self.tasks_pruned.saturating_add(other.tasks_pruned);
        self.tasks_with_suppressed_propagation = self
            .tasks_with_suppressed_propagation
            .saturating_add(other.tasks_with_suppressed_propagation);
    }

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
