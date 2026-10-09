use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;

use super::super::super::workflow_progression_state::WorthQueryWorkflowAdvanceStep;
use super::super::super::{
    WorthQueryAdmittedWorkflowStage, WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowRun,
    WorthQueryWorkflowStageComputePayload, WorthQueryWorkflowStageExecutorFailure,
    WorthQueryWorkflowValue,
};
use super::{ComputedWorkflowFrontier, WorkflowStageComputation, WorkflowStageComputationIdentity};

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
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        run: &mut WorthQueryWorkflowRun<D, O, F, L>,
        workspace: &mut WorthQueryWorkspace,
        mut admitted: Option<WorthQueryAdmittedWorkflowStage>,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        assert_eq!(
            self.owner,
            run.identity(),
            "a sealed frontier belongs to one workflow run"
        );
        assert_eq!(
            self.order.as_slice().len(),
            self.members.len(),
            "a sealed frontier has complete membership"
        );
        // The order proof travels intact from membership to application. Neither
        // computation completion order nor host callbacks can select this order.
        let mut members = self.members;
        for stage in self.order.into_keys() {
            let member = members
                .remove(&stage)
                .expect("sealed membership owns every canonical key");
            let expected = WorkflowStageComputationIdentity {
                frontier: self.identity.clone(),
                stage: stage.clone(),
            };
            let result = match member.computation {
                WorkflowStageComputation::Ready(computed) => computed.into_result(&expected),
                WorkflowStageComputation::Failed(failure) => Err(failure),
                WorkflowStageComputation::Denied(kind) => return Err(run.denial(kind)),
                WorkflowStageComputation::Unstarted => {
                    unreachable!("application stops at the preparation failure")
                }
            }
            .map_err(|failure| failure.into_executor_failure());
            let slot = CanonicalWorkflowStageResult {
                stage,
                input: member.input,
                result,
            };
            let step = if let Some(admitted) = admitted.take() {
                run.advance_once_with_admitted_computation(execution, admitted, slot, workspace)?
            } else {
                run.advance_once_with_computation(execution, slot, workspace)?
            };
            if matches!(step, WorthQueryWorkflowAdvanceStep::Deferred(_)) {
                return Ok(step);
            }
        }
        // A failed application returns above; dropping the iterator drops the
        // whole suffix. No suffix material, receipt or charge reaches the run.
        Ok(WorthQueryWorkflowAdvanceStep::Advanced)
    }
}
