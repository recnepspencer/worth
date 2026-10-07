use crate::data::error::SignalError;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::{EvaluationPlan, ExecutionReport};

use super::super::super::state::SignalRuntime;
use super::super::shared::{
    absorb_execution_report_telemetry, apply_strategy_maintenance, execute_plan_with_runtime_config,
};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn execute_prepared_plan<F, O>(
        &mut self,
        plan: &EvaluationPlan,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let strategy = self.derive_evaluation_strategy();
        let report = self.execute_prepared_plan_serial(plan, runtime_ctx, evaluator)?;
        apply_strategy_maintenance(&mut self.graph, strategy);
        Ok(report)
    }

    fn execute_prepared_plan_serial<F, O>(
        &mut self,
        plan: &EvaluationPlan,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.admit_temporal_wakes_for_plan(plan)?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal_lowering = self.temporal_lowering_context_for_plan(plan);
        let serial = self.graph.bounded_serial_request();
        let report = execute_plan_with_runtime_config(
            &mut self.graph,
            &self.config,
            temporal_lowering,
            runtime_ctx,
            plan,
            evaluator,
            worth_execution::ExecutionRequest::serial(&serial),
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
