use crate::data::error::SignalError;
use crate::data::error::SignalPublicationProgress;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::precompute::callback::CheckedPrecompute;
use crate::logic::planner::run_signal_request_scope;
use crate::logic::planner::ExecutionReport;
use worth_execution::ExecutionResourceLease;
use worth_execution::MapKernelContext;

use super::super::super::state::SignalRuntime;
use super::super::shared::{
    absorb_execution_report_telemetry, apply_strategy_maintenance,
    execute_targets_with_prepared_runtime_config_in_scope, execute_targets_with_runtime_config,
};

use super::request::ExecutionIntent;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn evaluate_checked<F, O>(
        &mut self,
        targets: &[crate::data::handle::NodeId],
        request_mode: EvaluationRequestMode,
        runtime_ctx: &Ctx,
        evaluator: &F,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'graph, 'work, 'run, 'lease> Fn(
                &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
            ) -> Result<O, SignalError>
            + Sync,
        O: IntoEvaluationOutput,
    {
        let result = run_signal_request_scope(lease, |work, disposition, preparation| {
            self.execute_checked_targets_in_scope(
                targets,
                request_mode,
                runtime_ctx,
                evaluator,
                lease,
                work,
                disposition,
                preparation,
            )
        });
        self.record_checked_result(result)
    }

    pub fn evaluate_dirty_checked<F, O>(
        &mut self,
        runtime_ctx: &Ctx,
        evaluator: &F,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'graph, 'work, 'run, 'lease> Fn(
                &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
            ) -> Result<O, SignalError>
            + Sync,
        O: IntoEvaluationOutput,
    {
        let result = run_signal_request_scope(lease, |work, disposition, preparation| {
            crate::logic::planner::precompute::work::checkpoint(
                Some(&mut *work),
                self.graph.active_node_count().saturating_add(1),
            )?;
            preparation.claim_vec::<crate::data::handle::NodeId>(self.graph.active_node_count())?;
            let targets = crate::logic::transaction::helpers::collect_dirty_targets(&self.graph);
            if targets.is_empty() {
                return Ok(crate::logic::transaction::helpers::empty_execution_report());
            }
            self.execute_checked_targets_in_scope(
                &targets,
                EvaluationRequestMode::Default,
                runtime_ctx,
                evaluator,
                lease,
                work,
                disposition,
                preparation,
            )
        });
        self.record_checked_result(result)
    }

    fn execute_checked_targets_in_scope<F, O>(
        &mut self,
        targets: &[crate::data::handle::NodeId],
        request_mode: EvaluationRequestMode,
        runtime_ctx: &Ctx,
        evaluator: &F,
        lease: &ExecutionResourceLease<'_>,
        work: &mut MapKernelContext<'_, '_>,
        disposition: &mut SignalPublicationProgress,
        preparation: &mut SignalPreparationBudget,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'graph, 'work, 'run, 'lease> Fn(
                &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
            ) -> Result<O, SignalError>
            + Sync,
        O: IntoEvaluationOutput,
    {
        crate::logic::planner::precompute::work::checkpoint(
            Some(&mut *work),
            self.graph.active_node_count().saturating_add(targets.len()),
        )?;
        preparation.claim_vec::<crate::data::handle::NodeId>(targets.len())?;
        self.admit_temporal_wakes_for_nodes(targets)?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal = self.temporal_lowering_context_for_nodes(targets);
        let report = execute_targets_with_prepared_runtime_config_in_scope(
            &mut self.graph,
            &self.config,
            temporal,
            targets,
            request_mode,
            &CheckedPrecompute::new(runtime_ctx, evaluator),
            lease,
            work,
            disposition,
            preparation,
        )?;
        self.retire_consumed_temporal_wakes_from_report(&report)?;
        Ok(report)
    }

    fn record_checked_result(
        &mut self,
        result: Result<ExecutionReport, SignalError>,
    ) -> Result<ExecutionReport, SignalError> {
        match &result {
            Ok(report) => {
                let physical = report.execution.last().copied();
                self.graph.record_completed_checked_execution(report);
                self.telemetry.execution.last_execution_report = physical;
                if self.graph.captures_observation_surface(
                    crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
                ) {
                    absorb_execution_report_telemetry(&mut self.telemetry, report);
                }
            }
            Err(SignalError::ExecutionStopped(stop)) => {
                let physical = stop.execution();
                self.graph.with_telemetry(|telemetry| {
                    telemetry.execution.last_execution_report = Some(physical)
                });
                self.telemetry.execution.last_execution_report = Some(physical);
            }
            Err(_) => {
                self.telemetry.execution.last_execution_report = None;
            }
        }
        result
    }

    pub fn evaluate_dirty<F, O>(
        &mut self,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let strategy = self.derive_evaluation_strategy();
        let report =
            self.execute_evaluation(ExecutionIntent::Dirty, runtime_ctx, evaluator, None)?;
        apply_strategy_maintenance(&mut self.graph, strategy);
        Ok(report)
    }

    pub(super) fn execute_evaluation<F, O>(
        &mut self,
        intent: ExecutionIntent<'_>,
        runtime_ctx: &Ctx,
        evaluator: &F,
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let owned_targets;
        let (targets, request_mode) = match intent {
            ExecutionIntent::Targets {
                targets,
                request_mode,
            } => (targets, request_mode),
            ExecutionIntent::Dirty => {
                owned_targets =
                    crate::logic::transaction::helpers::collect_dirty_targets(&self.graph);
                if owned_targets.is_empty() {
                    return Ok(crate::logic::transaction::helpers::empty_execution_report());
                }
                (&owned_targets[..], EvaluationRequestMode::Default)
            }
        };
        self.admit_temporal_wakes_for_nodes(targets)?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal_lowering = self.temporal_lowering_context_for_nodes(targets);
        let report = execute_targets_with_runtime_config(
            &mut self.graph,
            &self.config,
            temporal_lowering,
            runtime_ctx,
            targets,
            request_mode,
            evaluator,
            lease,
        )?;
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            absorb_execution_report_telemetry(&mut self.telemetry, &report);
        }
        self.retire_consumed_temporal_wakes_from_report(&report)?;
        Ok(report)
    }
}
