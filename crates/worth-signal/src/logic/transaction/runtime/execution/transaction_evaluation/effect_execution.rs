use crate::clock::RuntimeInstant;
use crate::data::error::SignalError;
use crate::diagnostics::ExecutionFailurePhase;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::{EvaluationPlan, ExecutionReport};

use super::super::super::transaction::SignalTransaction;
use super::super::shared::{absorb_execution_report_telemetry, execute_plan_with_runtime_config};

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn execute_prepared_plan<F, O>(
        &mut self,
        plan: &EvaluationPlan,
        evaluator: &F,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        for stage in &plan.stages {
            self.stage_task_candidates(&stage.tasks)?;
        }
        self.admit_temporal_wakes_for_plan(plan)?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal_lowering = self.temporal_lowering_context_for_plan(plan);
        let execution_start = RuntimeInstant::now();
        let report = match execute_plan_with_runtime_config(
            self.graph,
            self.config,
            temporal_lowering,
            &*self.runtime_ctx,
            plan,
            evaluator,
            self.execution,
        ) {
            Ok(report) => report,
            Err(err) => {
                self.record_failure_from_error(
                    ExecutionFailurePhase::Apply,
                    &err,
                    Some(plan.summary),
                );
                return Err(err);
            }
        };
        self.execution_state
            .record_report(&report, execution_start.elapsed().as_nanos());
        self.scratch.temporal.absorb_report(&report);
        self.lower_observation_classifications_from_report(&report)?;
        self.with_telemetry(|telemetry| absorb_execution_report_telemetry(telemetry, &report));
        self.retire_consumed_temporal_wakes_from_report(&report)?;
        Ok(report)
    }
}
