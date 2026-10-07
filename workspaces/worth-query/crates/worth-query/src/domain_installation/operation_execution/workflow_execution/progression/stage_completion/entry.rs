//! Request-carrying singleton entry and its outcome routing.
use super::*;
impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    /// Every entry carries its caller's execution request.
    /// ```
    /// use worth_query::facade::{domain::{WorthQueryWorkflowRun, WorthQueryWorkflowValue}, foundation::MutationPreparationLaneWitness, runtime::{WorthQueryWorkspace, ExecutionRequest}};
    /// fn enter(run: WorthQueryWorkflowRun<(), (), (), MutationPreparationLaneWitness>, workspace: &mut WorthQueryWorkspace, request: ExecutionRequest<'_, '_>) {
    ///     let _ = run.advance("member", WorthQueryWorkflowValue::NotRequired, workspace , request);
    /// }
    /// ```
    /// ```compile_fail
    /// use worth_query::facade::{domain::{WorthQueryWorkflowRun, WorthQueryWorkflowValue}, foundation::MutationPreparationLaneWitness, runtime::{WorthQueryWorkspace, ExecutionRequest}};
    /// fn enter(run: WorthQueryWorkflowRun<(), (), (), MutationPreparationLaneWitness>, workspace: &mut WorthQueryWorkspace, request: ExecutionRequest<'_, '_>) {
    ///     let _ = run.advance("member", WorthQueryWorkflowValue::NotRequired, workspace);
    /// }
    /// ```
    pub fn advance(
        mut self,
        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        let runtime_admission = match self.admit_stage_runtime_authority(workspace) {
            Ok(admission) => admission,
            Err(denial) => return self.outcome_from_denial(denial),
        };
        self.advance_with_runtime_admission(
            stage_identity,
            input,
            workspace,
            runtime_admission,
            request,
        )
    }

    pub(in crate::domain_installation::operation_execution) fn advance_with_runtime_admission(
        mut self,
        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
        runtime_admission: WorthQueryWorkflowStageRuntimeAdmission,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        match self.advance_once_with_runtime_admission(
            stage_identity,
            input,
            workspace,
            runtime_admission,
            request,
        ) {
            Ok(WorthQueryWorkflowAdvanceStep::Advanced) => TransitionOutcome::Success(self),
            Ok(WorthQueryWorkflowAdvanceStep::Deferred(conditional)) => {
                TransitionOutcome::Deferred(
                    crate::domain_installation::WorthQueryDeferredWorkflowStage {
                        run: self,
                        conditional,
                    },
                )
            }
            Err(denial) => self.outcome_from_denial(denial),
        }
    }
}
