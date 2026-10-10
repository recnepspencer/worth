use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::summary::EvaluationPlanSummary;
use crate::logic::planner::precompute::callback::SignalPrecompute;
use crate::logic::planner::precompute::TemporalLoweringContext;
use worth_execution::MapKernelContext;

use super::super::types::{ExecutionReport, PlanSummary, ResolvedSignalPlannerPolicy};
use super::diagnostics::record_successful_execution;
use super::reporting::begin_execution_report;
use crate::data::request_preparation::SignalPreparationBudget;

pub(crate) struct ExecutionContext<'a, 'work, 'run, 'request, P, R>
where
    P: SignalPrecompute,
    R: ComparatorPolicyResolver,
{
    pub(crate) graph: &'a mut SignalGraph,
    pub(crate) summary: &'a PlanSummary,
    pub(crate) precompute: &'a P,
    pub(crate) comparator_resolver: &'a mut R,
    pub(crate) temporal_lowering: TemporalLoweringContext,
    pub(crate) request: worth_execution::ExecutionRequest<'a, 'request>,
    pub(crate) policy: ResolvedSignalPlannerPolicy,
    pub(crate) request_work: &'a mut MapKernelContext<'work, 'run>,
    pub(crate) preparation: &'a mut SignalPreparationBudget,
    pub(crate) next_record_id: u64,
    pub(crate) next_segment_id: u64,
    pub(crate) reuse_origin_storage_claimed: bool,
    pub(crate) plan_summary: EvaluationPlanSummary,
    pub(crate) first_target: Option<NodeId>,
    pub(crate) report: ExecutionReport,
}

impl<'a, 'work, 'run, 'request, P, R> ExecutionContext<'a, 'work, 'run, 'request, P, R>
where
    P: SignalPrecompute,
    R: ComparatorPolicyResolver,
{
    pub(crate) fn new(
        graph: &'a mut SignalGraph,
        summary: &'a PlanSummary,
        stage_count: usize,
        maybe_stale_validation_tasks: u64,
        plan_summary: EvaluationPlanSummary,
        first_target: Option<NodeId>,
        precompute: &'a P,
        comparator_resolver: &'a mut R,
        temporal_lowering: TemporalLoweringContext,
        request: worth_execution::ExecutionRequest<'a, 'request>,
        policy: ResolvedSignalPlannerPolicy,
        request_work: &'a mut MapKernelContext<'work, 'run>,
        preparation: &'a mut SignalPreparationBudget,
    ) -> Self {
        let report =
            begin_execution_report(graph, summary, stage_count, maybe_stale_validation_tasks);
        Self {
            graph,
            summary,
            precompute,
            comparator_resolver,
            temporal_lowering,
            request,
            policy,
            request_work,
            preparation,
            next_record_id: 1,
            next_segment_id: 1,
            reuse_origin_storage_claimed: false,
            plan_summary,
            first_target,
            report,
        }
    }

    pub(crate) fn finish(self) -> ExecutionReport {
        record_successful_execution(
            self.graph,
            self.plan_summary,
            self.first_target,
            &self.report,
        );
        self.report
    }
}
