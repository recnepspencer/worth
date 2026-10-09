use crate::data::error::SignalError;
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::precompute::callback::{CheckedPrecompute, LegacyPrecompute};
use crate::logic::planner::{
    build_evaluation_plan, execute_prepared_plan, execute_prepared_plan_in_scope,
    execute_prepared_plan_with_precompute, run_signal_execution_request_scope, EvaluationPlan,
    ExecutionReport, TemporalLoweringContext,
};
use crate::logic::prepared::{ExecutionReadView, PreparedEvaluation};
use worth_execution::ExecutionRequest;

use super::graph::SignalGraph;

impl SignalGraph {
    pub fn evaluate_checked<Ctx, F, O>(
        &mut self,
        targets: &[crate::data::handle::NodeId],
        request_mode: EvaluationRequestMode,
        domain_ctx: &Ctx,
        evaluator: &F,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionReport, SignalError>
    where
        Ctx: Sync,
        F: for<'graph, 'work, 'run, 'lease> Fn(
                &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
            ) -> Result<O, SignalError>
            + Sync,
        O: IntoEvaluationOutput,
    {
        self.with_telemetry(|telemetry| telemetry.execution.last_execution_report = None);
        let mut comparator = crate::data::comparator::DefaultComparatorResolver;
        let mut resolver = crate::data::comparator::DefaultComparatorPolicyResolver {
            fallback: crate::data::comparator::VersionComparatorPolicy::Exact,
            custom: &mut comparator,
        };
        let result = run_signal_execution_request_scope(
            request,
            |work, disposition, preparation| {
                let plan = crate::logic::planner::planning::build_evaluation_plan_with_policy_resolver_and_work(
                    self, targets, request_mode, &mut resolver, Some(&mut *work), Some(&mut *preparation),
                )?;
                execute_prepared_plan_in_scope(
                    self,
                    &plan,
                    &CheckedPrecompute::new(domain_ctx, evaluator),
                    &mut resolver,
                    TemporalLoweringContext::graph_only(),
                    request,
                    work,
                    disposition,
                    preparation,
                )
            },
        );
        self.record_checked_execution_result(&result);
        result
    }

    fn record_checked_execution_result(&mut self, result: &Result<ExecutionReport, SignalError>) {
        match result {
            Ok(report) => self.record_completed_checked_execution(report),
            Err(error) => {
                let physical = match error {
                    SignalError::ExecutionStopped(stop) => Some(stop.execution()),
                    _ => None,
                };
                self.with_telemetry(|telemetry| {
                    telemetry.execution.last_execution_report = physical
                });
            }
        }
    }

    /// Stage and planner counters are written at their owning transitions.
    /// This request boundary records only usage and the enclosing report.
    pub(crate) fn record_completed_checked_execution(&mut self, report: &ExecutionReport) {
        let parallel = report.stages.iter().any(|stage| {
            stage.outcome == crate::logic::planner::StageExecutionOutcome::CompletedParallel
        });
        let physical = report.execution.last().copied();
        self.with_telemetry(|telemetry| {
            telemetry.execution.last_execution_report = physical;
            if parallel {
                telemetry.execution.parallel_executor_usage_count += 1;
            } else if physical.is_some() {
                telemetry.execution.serial_executor_usage_count += 1;
            }
        });
    }

    pub(crate) fn execute_prepared_plan_with_precompute<F>(
        &mut self,
        plan: &EvaluationPlan,
        precompute: &F,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: Fn(
                crate::data::handle::NodeId,
                &ExecutionReadView<'_>,
            ) -> Result<PreparedEvaluation, SignalError>
            + Sync,
    {
        let mut comparator = crate::data::comparator::DefaultComparatorResolver;
        let mut resolver = crate::data::comparator::DefaultComparatorPolicyResolver {
            fallback: crate::data::comparator::VersionComparatorPolicy::Exact,
            custom: &mut comparator,
        };
        let serial = self.bounded_serial_request();
        execute_prepared_plan_with_precompute(
            self,
            plan,
            &LegacyPrecompute::new(precompute),
            &mut resolver,
            TemporalLoweringContext::graph_only(),
            worth_execution::ExecutionRequest::serial(&serial),
        )
    }

    pub fn build_evaluation_plan(
        &mut self,
        targets: &[crate::data::handle::NodeId],
        request_mode: EvaluationRequestMode,
    ) -> Result<EvaluationPlan, SignalError> {
        build_evaluation_plan(self, targets, request_mode)
    }

    pub fn execute_prepared_plan<Ctx, F, O>(
        &mut self,
        plan: &EvaluationPlan,
        domain_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<ExecutionReport, SignalError>
    where
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        execute_prepared_plan(self, plan, domain_ctx, evaluator)
    }

    pub fn execute_prepared_plan_checked<Ctx, F, O>(
        &mut self,
        plan: &EvaluationPlan,
        domain_ctx: &Ctx,
        evaluator: &F,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionReport, SignalError>
    where
        Ctx: Sync,
        F: for<'graph, 'work, 'run, 'lease> Fn(
                &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
            ) -> Result<O, SignalError>
            + Sync,
        O: IntoEvaluationOutput,
    {
        self.with_telemetry(|telemetry| telemetry.execution.last_execution_report = None);
        let mut comparator = crate::data::comparator::DefaultComparatorResolver;
        let mut resolver = crate::data::comparator::DefaultComparatorPolicyResolver {
            fallback: crate::data::comparator::VersionComparatorPolicy::Exact,
            custom: &mut comparator,
        };
        let result = execute_prepared_plan_with_precompute(
            self,
            plan,
            &CheckedPrecompute::new(domain_ctx, evaluator),
            &mut resolver,
            TemporalLoweringContext::graph_only(),
            request,
        );
        self.record_checked_execution_result(&result);
        result
    }
}
