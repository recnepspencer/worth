//! One installed opening, lent through singleton preparation and publication.
use super::*;
impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub fn advance(
        self,
        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        let owner = workspace.advancement_owner();
        owner
            .with_advancement(|phase| {
                self.advance_in_advancement(&phase, stage_identity, input, workspace)
            })
            .unwrap_or_else(|cause| {
                TransitionOutcome::Denied(WorthQueryWorkflowAdvanceDenial::new(
                    WorthQueryWorkflowAdvanceDenialKind::ExecutionRequest(cause),
                    Default::default(),
                ))
            })
    }

    pub(in crate::domain_installation::operation_execution) fn advance_in_advancement(
        mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        let execution = phase;
        let runtime_admission = match self.admit_stage_runtime_authority(workspace) {
            Ok(admission) => admission,
            Err(denial) => return self.outcome_from_denial(denial),
        };
        self.advance_with_runtime_admission(
            execution,
            stage_identity,
            input,
            workspace,
            runtime_admission,
        )
    }

    pub(in crate::domain_installation::operation_execution) fn advance_with_runtime_admission(
        mut self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,

        stage_identity: &str,
        input: WorthQueryWorkflowValue,
        workspace: &mut WorthQueryWorkspace,
        runtime_admission: WorthQueryWorkflowStageRuntimeAdmission,
    ) -> WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        match self.advance_once_with_runtime_admission(
            execution,
            stage_identity,
            input,
            workspace,
            runtime_admission,
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
