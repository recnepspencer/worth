use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;

use super::super::super::workflow_progression_state::WorthQueryWorkflowAdvanceStep;
use super::super::super::WorthQueryWorkflowAdvanceDenialKind as Denial;
use super::super::super::{
    WorthQueryAdmittedWorkflowStage, WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowRun,
    WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageExecutorFailure,
    WorthQueryWorkflowValue,
};
use super::{ComputedWorkflowFrontier, WorkflowFrontierFailure, WorkflowStageComputationIdentity};

/// Only consuming the complete sealed batch issues these canonical slots.
pub(in crate::domain_installation::operation_execution) struct CanonicalWorkflowStageResult {
    stage: String,
    input: WorthQueryWorkflowValue,
    result: Result<WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageExecutorFailure>,
}

impl CanonicalWorkflowStageResult {
    pub(in crate::domain_installation::operation_execution) fn stage_identity(&self) -> &str {
        &self.stage
    }
    pub(in crate::domain_installation::operation_execution) fn input(
        &self,
    ) -> &WorthQueryWorkflowValue {
        &self.input
    }
    pub(in crate::domain_installation::operation_execution) fn into_parts(
        self,
    ) -> (
        WorthQueryWorkflowValue,
        Result<WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageExecutorFailure>,
    ) {
        (self.input, self.result)
    }
}

impl ComputedWorkflowFrontier {
    pub(in crate::domain_installation::operation_execution) fn apply<
        D: 'static,
        O: 'static,
        F: 'static,
        L: BasisOperationLane,
    >(
        self,
        run: &mut WorthQueryWorkflowRun<D, O, F, L>,
        workspace: &mut WorthQueryWorkspace,
        mut admitted: Option<WorthQueryAdmittedWorkflowStage>,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        if self.owner != run.identity() {
            return Err(run.denial(Denial::ParallelFrontierShape));
        }
        let Some(charged_work) = run
            .counters
            .computation_charged_work
            .checked_add(self.charged_work)
        else {
            return Err(run.denial(Denial::ComputationWorkExhausted {
                stage_identity: None,
                cause: worth_execution::MapKernelStop::WorkCounterOverflow,
            }));
        };
        run.counters.computation_charged_work = charged_work;
        let mut prefix_len = 0;
        for member in self.prefix {
            if member.rank != prefix_len {
                return Err(run.denial(Denial::ParallelFrontierShape));
            }
            let Some(stage) = self.order.as_slice().get(member.rank) else {
                return Err(run.denial(Denial::ParallelFrontierShape));
            };
            let expected = WorkflowStageComputationIdentity {
                frontier: self.identity.clone(),
                stage: stage.clone(),
            };
            let slot = CanonicalWorkflowStageResult {
                stage: stage.clone(),
                input: member.input,
                result: member
                    .computed
                    .into_result(&expected)
                    .map_err(|failure| failure.into_executor_failure()),
            };
            let step = if let Some(admitted) = admitted.take() {
                run.advance_once_with_admitted_computation(admitted, slot, workspace)?
            } else {
                run.advance_once_with_computation(slot, workspace)?
            };
            if matches!(step, WorthQueryWorkflowAdvanceStep::Deferred(_)) {
                return Ok(step);
            }
            prefix_len += 1;
        }
        if let Some(stop) = self.stop {
            let Some(rank) = stop.rank else {
                return match stop.failure {
                    WorkflowFrontierFailure::Denied(kind) => Err(run.denial(kind)),
                    WorkflowFrontierFailure::Domain(_)
                    | WorkflowFrontierFailure::Preparation(_) => {
                        Err(run.denial(Denial::ParallelFrontierShape))
                    }
                };
            };
            if rank != prefix_len {
                return Err(run.denial(Denial::ParallelFrontierShape));
            }
            let (Some(stage), Some(input)) = (self.order.as_slice().get(rank), stop.input) else {
                return Err(run.denial(Denial::ParallelFrontierShape));
            };
            let result = match stop.failure {
                WorkflowFrontierFailure::Domain(computed) => {
                    let expected = WorkflowStageComputationIdentity {
                        frontier: self.identity,
                        stage: stage.clone(),
                    };
                    computed.into_result(&expected)
                }
                WorkflowFrontierFailure::Preparation(failure) => Err(failure),
                WorkflowFrontierFailure::Denied(kind) => {
                    // Even a preparation refusal crosses ordinary runtime and
                    // stage admission, preserving the ordinary denial counters.
                    if admitted.take().is_none() {
                        let runtime = run.admit_stage_runtime_authority(workspace)?;
                        run.admit_stage(stage, &input, runtime)?;
                    }
                    return Err(run.denial(kind));
                }
            }
            .map_err(|failure| failure.into_executor_failure());
            let slot = CanonicalWorkflowStageResult {
                stage: stage.clone(),
                input,
                result,
            };
            return if let Some(admitted) = admitted.take() {
                run.advance_once_with_admitted_computation(admitted, slot, workspace)
            } else {
                run.advance_once_with_computation(slot, workspace)
            };
        }
        if prefix_len != self.order.as_slice().len() {
            return Err(run.denial(Denial::ParallelFrontierShape));
        }
        Ok(WorthQueryWorkflowAdvanceStep::Advanced)
    }
}
