use crate::clock::RuntimeInstant;
use crate::data::comparator::{
    DefaultComparatorPolicyResolver, DefaultComparatorResolver, VersionComparatorPolicy,
};
use crate::data::error::{SignalError, SignalPublicationProgress};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::diagnostics::ExecutionFailurePhase;
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::{EvaluationRequestMode, IntoEvaluationOutput};
use crate::logic::planner::precompute::callback::CheckedPrecompute;
use crate::logic::planner::{
    admit_direct_task_with_policy_resolver, run_signal_request_scope, ExecutionReport,
};
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use super::super::super::transaction::SignalTransaction;
use super::super::shared::{
    absorb_execution_report_telemetry, execute_targets_with_prepared_runtime_config_in_scope,
    execute_targets_with_runtime_config_detailed,
};

use super::request::TransactionExecutionIntent;

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
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
                false,
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
            let targets = self.collect_dirty_targets();
            if targets.is_empty() {
                return Ok(crate::logic::transaction::helpers::empty_execution_report());
            }
            self.execute_checked_targets_in_scope(
                &targets,
                EvaluationRequestMode::Default,
                true,
                evaluator,
                lease,
                work,
                disposition,
                preparation,
            )
        });
        self.record_checked_result(result)
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_checked_targets_in_scope<F, O>(
        &mut self,
        targets: &[crate::data::handle::NodeId],
        request_mode: EvaluationRequestMode,
        stage_task_candidates: bool,
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
        if stage_task_candidates {
            let mut comparator = DefaultComparatorResolver;
            let mut resolver = DefaultComparatorPolicyResolver {
                fallback: VersionComparatorPolicy::Exact,
                custom: &mut comparator,
            };
            preparation.claim_vec::<crate::logic::planner::EligibleTask>(targets.len())?;
            let candidates = targets
                .iter()
                .copied()
                .map(|node| {
                    admit_direct_task_with_policy_resolver(
                        &*self.graph,
                        node,
                        request_mode,
                        &mut resolver,
                    )
                })
                .collect::<Result<Vec<_>, SignalError>>()?;
            self.stage_task_candidates(&candidates)?;
        } else {
            self.stage_evaluate_candidate_batch(targets)?;
        }
        self.admit_temporal_wakes_for_nodes(targets)?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal = self.temporal_lowering_context_for_nodes(targets);
        let started = RuntimeInstant::now();
        let report = execute_targets_with_prepared_runtime_config_in_scope(
            self.graph,
            self.config,
            temporal,
            targets,
            request_mode,
            &CheckedPrecompute::new(&*self.runtime_ctx, evaluator),
            lease,
            work,
            disposition,
            preparation,
        )?;
        self.execution_state
            .record_report(&report, started.elapsed().as_nanos());
        self.scratch.temporal.absorb_report(&report);
        self.lower_observation_classifications_from_report(&report)?;
        self.retire_consumed_temporal_wakes_from_report(&report)?;
        Ok(report)
    }

    fn record_checked_result(
        &mut self,
        result: Result<ExecutionReport, SignalError>,
    ) -> Result<ExecutionReport, SignalError> {
        match &result {
            Ok(report) => {
                self.graph.record_completed_checked_execution(report);
                self.with_telemetry(|telemetry| {
                    absorb_execution_report_telemetry(telemetry, report)
                });
            }
            Err(SignalError::ExecutionStopped(stop)) => {
                let physical = stop.execution();
                self.graph.with_telemetry(|telemetry| {
                    telemetry.execution.last_execution_report = Some(physical)
                });
                self.with_telemetry(|telemetry| {
                    telemetry.execution.last_execution_report = Some(physical)
                });
                self.record_failure_from_error(
                    ExecutionFailurePhase::Apply,
                    result.as_ref().unwrap_err(),
                    None,
                );
            }
            Err(error) => self.record_failure_from_error(ExecutionFailurePhase::Apply, error, None),
        }
        result
    }

    pub(super) fn execute_evaluation<F, O>(
        &mut self,
        intent: TransactionExecutionIntent<'_>,
        evaluator: &F,
        lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let owned_targets;
        let (targets, request_mode) = match intent {
            TransactionExecutionIntent::Targets {
                targets,
                request_mode,
                stage_task_candidates,
            } => {
                if stage_task_candidates {
                    let mut comparator = DefaultComparatorResolver;
                    let mut resolver = DefaultComparatorPolicyResolver {
                        fallback: VersionComparatorPolicy::Exact,
                        custom: &mut comparator,
                    };
                    let stage_targets = targets
                        .iter()
                        .copied()
                        .map(|node| {
                            admit_direct_task_with_policy_resolver(
                                &*self.graph,
                                node,
                                request_mode,
                                &mut resolver,
                            )
                        })
                        .collect::<Result<Vec<_>, SignalError>>()?;
                    self.stage_task_candidates(&stage_targets)?;
                } else {
                    self.stage_evaluate_candidate_batch(targets)?;
                }
                (targets, request_mode)
            }
            TransactionExecutionIntent::Dirty => {
                owned_targets = self.collect_dirty_targets();
                if owned_targets.is_empty() {
                    return Ok(crate::logic::transaction::helpers::empty_execution_report());
                }
                let mut comparator = DefaultComparatorResolver;
                let mut resolver = DefaultComparatorPolicyResolver {
                    fallback: VersionComparatorPolicy::Exact,
                    custom: &mut comparator,
                };
                let stage_targets = owned_targets
                    .iter()
                    .copied()
                    .map(|node| {
                        admit_direct_task_with_policy_resolver(
                            &*self.graph,
                            node,
                            EvaluationRequestMode::Default,
                            &mut resolver,
                        )
                    })
                    .collect::<Result<Vec<_>, SignalError>>()?;
                self.stage_task_candidates(&stage_targets)?;
                (&owned_targets[..], EvaluationRequestMode::Default)
            }
        };

        self.admit_temporal_wakes_for_nodes(targets)?;
        self.promote_due_temporal_wakes_ready()?;
        let temporal_lowering = self.temporal_lowering_context_for_nodes(targets);
        let execution_start = RuntimeInstant::now();
        let report = match execute_targets_with_runtime_config_detailed(
            self.graph,
            self.config,
            temporal_lowering,
            &*self.runtime_ctx,
            targets,
            request_mode,
            evaluator,
            lease,
        ) {
            Ok(report) => report,
            Err(failure) => {
                let err = failure.error;
                self.record_failure_from_error(
                    ExecutionFailurePhase::Apply,
                    &err,
                    Some(failure.plan_summary),
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
