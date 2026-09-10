use crate::data::proof::{InvalidationPlanningEstimate, InvalidationTraceRecord};
use crate::diagnostics::failure::{FailureSummary, RollbackDiagnostic};
use crate::diagnostics::flow::RetainedFlowSummaryView;
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::GraphSummary;
use crate::logic::transaction::ObservationBoundarySummary;

use super::GraphHealthDiagnostics;

impl<'a> GraphHealthDiagnostics<'a> {
    pub fn summary(&self, profile: DiagnosticsTier) -> GraphSummary {
        self.graph.observe().diagnostics_summary(profile)
    }

    pub fn summary_now(&self) -> GraphSummary {
        self.summary(self.graph.installed_runtime_policy().tier())
    }

    pub fn current(&self, profile: DiagnosticsTier) -> GraphSummary {
        self.summary(profile)
    }

    pub fn current_now(&self) -> GraphSummary {
        self.summary_now()
    }

    pub fn latest_flow(&self) -> Option<RetainedFlowSummaryView<'a>> {
        self.graph.observe().latest_flow_diagnostics()
    }

    pub fn latest_failure(&self) -> Option<&'a FailureSummary> {
        self.graph.observe().latest_failure_diagnostics()
    }

    pub fn latest_rollback(&self) -> Option<&'a RollbackDiagnostic> {
        self.graph.observe().latest_rollback_diagnostics()
    }

    pub fn latest_observation(&self) -> Option<&'a ObservationBoundarySummary> {
        self.graph.observe().latest_observation_summary()
    }

    pub fn latest_invalidation_planning_estimate(
        &self,
    ) -> Option<&'a InvalidationPlanningEstimate> {
        self.graph.observe().latest_invalidation_planning_estimate()
    }

    pub fn latest_invalidation_trace_records(&self) -> &'a [InvalidationTraceRecord] {
        self.graph.observe().latest_invalidation_trace_records()
    }

    pub fn recent_history(&self) -> crate::diagnostics::summary::RetainedExecutionHistoryView<'a> {
        self.graph.observe().recent_execution_history_diagnostics()
    }
}
