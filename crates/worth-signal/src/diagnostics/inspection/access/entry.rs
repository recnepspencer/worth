use crate::data::handle::NodeId;
#[cfg(test)]
use crate::data::proof::FrontierDiagnosticsSidecar;
use crate::data::proof::{InvalidationPlanningEstimate, InvalidationTraceRecord};
use crate::diagnostics::compare::{
    explanations_semantically_equivalent, graphs_semantically_equivalent,
    plans_semantically_equivalent, repeat_run_summaries_equal, reports_semantically_equivalent,
    serial_parallel_reports_equivalent,
};
use crate::diagnostics::failure::{FailureSummary, RollbackDiagnostic};
use crate::diagnostics::flow::{FlowSummary, RetainedFlowSummaryView};
use crate::diagnostics::history::{
    inspect_execution, inspect_flow, inspect_graph, inspect_plan, inspect_report,
    ExecutionInspector, FlowInspector, GraphInspector, PlanInspector, ReportInspector,
};
use crate::diagnostics::profile::DiagnosticsTier;
use crate::diagnostics::summary::{
    EvaluationPlanSummary, ExecutionHistorySummary, ExecutionReportSummary, ExplanationSummary,
    GraphSummary,
};
use crate::logic::explain::NodeExplanation;
use crate::logic::planner::{EvaluationPlan, ExecutionReport};
use crate::logic::transaction::ObservationBoundarySummary;

use super::{
    GraphComparisonDiagnostics, GraphDiagnostics, GraphForensicDiagnostics, GraphHealthDiagnostics,
    GraphInspectDiagnostics,
};

impl<'a> GraphDiagnostics<'a> {
    pub fn forensic(&self) -> GraphForensicDiagnostics<'a> {
        GraphForensicDiagnostics { graph: self.graph }
    }

    pub fn compare(&self) -> GraphComparisonDiagnostics {
        GraphComparisonDiagnostics
    }

    /// Group health-oriented reads in one place.
    pub fn health_view(&self) -> GraphHealthDiagnostics<'a> {
        GraphHealthDiagnostics { graph: self.graph }
    }

    /// Group inspector-style reads in one place.
    pub fn inspect(&self) -> GraphInspectDiagnostics<'a> {
        GraphInspectDiagnostics { graph: self.graph }
    }

    pub fn explain(
        &self,
        node: NodeId,
    ) -> Result<NodeExplanation, crate::data::error::SignalError> {
        self.graph.observe().explain(node)
    }

    pub fn why(&self, node: NodeId) -> Result<NodeExplanation, crate::data::error::SignalError> {
        self.explain(node)
    }

    pub fn summary(&self, profile: DiagnosticsTier) -> GraphSummary {
        self.graph.observe().diagnostics_summary(profile)
    }

    pub fn summary_now(&self) -> GraphSummary {
        self.summary(self.graph.installed_runtime_policy().tier())
    }

    pub fn history(&self, profile: DiagnosticsTier) -> ExecutionHistorySummary {
        self.graph.observe().execution_history_summary(profile)
    }

    pub fn history_now(&self) -> ExecutionHistorySummary {
        self.history(self.graph.installed_runtime_policy().tier())
    }

    pub fn health(&self, profile: DiagnosticsTier) -> GraphSummary {
        self.summary(profile)
    }

    pub fn health_now(&self) -> GraphSummary {
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

    #[cfg(test)]
    pub(crate) fn latest_frontier_execution(&self) -> Option<&'a FrontierDiagnosticsSidecar> {
        self.graph.observe().latest_frontier_execution_summary()
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

    pub fn inspect_graph(&self) -> GraphInspector<'a> {
        inspect_graph(self.graph)
    }

    pub fn inspect_execution(&self) -> ExecutionInspector<'a> {
        inspect_execution(self.graph)
    }

    pub fn inspect_plan(&self, plan: &'a EvaluationPlan) -> PlanInspector<'a> {
        inspect_plan(plan)
    }

    pub fn inspect_report(&self, report: &'a ExecutionReport) -> ReportInspector<'a> {
        inspect_report(report)
    }

    pub fn inspect_flow(&self, flow: &'a FlowSummary) -> FlowInspector<'a> {
        inspect_flow(flow)
    }

    pub fn graphs_semantically_equivalent(
        &self,
        left: &GraphSummary,
        right: &GraphSummary,
    ) -> bool {
        graphs_semantically_equivalent(left, right)
    }

    pub fn plans_semantically_equivalent(
        &self,
        left: &EvaluationPlanSummary,
        right: &EvaluationPlanSummary,
    ) -> bool {
        plans_semantically_equivalent(left, right)
    }

    pub fn reports_semantically_equivalent(
        &self,
        left: &ExecutionReportSummary,
        right: &ExecutionReportSummary,
    ) -> bool {
        reports_semantically_equivalent(left, right)
    }

    pub fn explanations_semantically_equivalent(
        &self,
        left: &ExplanationSummary,
        right: &ExplanationSummary,
    ) -> bool {
        explanations_semantically_equivalent(left, right)
    }

    pub fn serial_parallel_reports_equivalent(
        &self,
        left: &ExecutionReportSummary,
        right: &ExecutionReportSummary,
    ) -> bool {
        serial_parallel_reports_equivalent(left, right)
    }

    pub fn repeat_run_summaries_equal<T>(&self, summaries: &[T]) -> bool
    where
        T: PartialEq,
    {
        repeat_run_summaries_equal(summaries)
    }
}
