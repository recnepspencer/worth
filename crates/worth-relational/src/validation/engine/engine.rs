use crate::validation::engine::InvariantRuntimeView;
use crate::validation::execution::InvariantStage;
use crate::validation::reduction::ReducedInvariantPrefix;
use worth_execution::{ExecutionResourceLease, ExecutionWorkCeiling};

use super::request::InvariantExecutionRequest;
use super::result::InvariantExecutionResult;

pub(crate) struct InvariantEngine<'runtime> {
    runtime: InvariantRuntimeView<'runtime>,
}

impl<'runtime> InvariantEngine<'runtime> {
    pub(crate) fn from_view(runtime: &InvariantRuntimeView<'runtime>) -> Self {
        Self {
            runtime: runtime.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new(runtime: &'runtime crate::runtime::RelationalRuntime) -> Self {
        Self {
            runtime: InvariantRuntimeView::from_runtime(runtime),
        }
    }

    pub(crate) fn execute<'state>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: InvariantExecutionRequest<'state>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        check_live(*execution)?;
        let lease = crate::authority::commit::pipeline::request_lease(execution);
        if lease.is_some_and(|lease| {
            lease.resolved_posture() == worth_foundational::ExecutionPosture::Automatic
        }) && self
            .runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .filter(|registration| request.includes_custom_registration(registration))
            .any(|registration| !registration.supports_checked_preparation())
        {
            return Err(
                crate::transactions::data::TransactionCommitError::execution(
                    crate::transactions::data::CommitExecutionDenial {
                        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                            crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel,
                        ),
                        partition_identity: None,
                    },
                ),
            );
        }
        // An absent accounting contract stays absent. Already active accounted
        // ancestors still govern the same inner patterns and reservations.
        if !execution.has_accounting() {
            return self.execute_inner(*execution, request);
        }
        // One original scope spans both stages even when no preparation account
        // is attached to the runtime view. Do not bill its aggregate report again.
        execution
            .run(ExecutionWorkCeiling::new(execution.work_ceiling()), |_| {
                self.execute_inner(*execution, request)
            })
            .map(|(result, _)| result)
            .map_err(request_failure)?
    }

    fn execute_inner<'state>(
        &self,
        execution: worth_execution::ExecutionRequest<'state, 'state>,
        request: InvariantExecutionRequest<'state>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError>
    where
        'runtime: 'state,
    {
        check_live(execution)?;
        let lease = crate::authority::commit::pipeline::request_lease(&execution);
        let mut runtime = self.runtime.with_candidate_input_plan(&request);
        runtime.custom_kernel_posture = lease.map_or(
            worth_foundational::ExecutionPosture::Serial,
            ExecutionResourceLease::resolved_posture,
        );
        let mut prefix = ReducedInvariantPrefix::new();
        if let Some(stage) = super::stage_execution::execute_stage(
            &runtime,
            execution,
            &request,
            lease,
            InvariantStage::Native,
        )? {
            let reduced = prefix.absorb_in_request(
                stage,
                execution,
                lease,
                runtime.commit_work_budget.as_ref(),
            );
            check_live(execution)?;
            prefix = reduced?;
        }
        check_live(execution)?;
        if !prefix.prevents_custom(request.execution_point()) {
            let selection = InvariantStage::Custom {
                packet_offset: prefix.packet_count(),
            };
            if let Some(stage) = super::stage_execution::execute_stage(
                &runtime, execution, &request, lease, selection,
            )? {
                let reduced = prefix.absorb_in_request(
                    stage,
                    execution,
                    lease,
                    runtime.commit_work_budget.as_ref(),
                );
                check_live(execution)?;
                prefix = reduced?;
            }
        }
        let completed = prefix.finish_in_request(
            &request,
            execution,
            lease,
            runtime.commit_work_budget.as_ref(),
        );
        check_live(execution)?;
        let completed = completed?;
        if let Some(inputs) = runtime.shared_candidate_inputs() {
            runtime
                .performance_access()
                .count_custom_invariant_candidate_inputs(inputs.counters());
        }
        if !completed.conflicts().is_empty() {
            self.runtime
                .performance_access()
                .count_preparation_reducer_conflicts(completed.conflicts().len());
        }
        check_live(execution)?;
        Ok(completed.into_result())
    }
}

pub(crate) fn check_live(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    execution.check_live().map_err(request_failure)
}

fn request_failure(
    stop: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> crate::transactions::data::TransactionCommitError {
    crate::transactions::data::TransactionCommitError::execution(
        crate::transactions::data::CommitExecutionDenial {
            kind: crate::transactions::data::CommitExecutionDenialKind::Cause(stop.into()),
            partition_identity: None,
        },
    )
}
