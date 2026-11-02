//! Readmit installed lifecycle programs while returning their original binding on refusal.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_elevation_request_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        WorthQueryElevationRequestOutcome,
    > {
        if let Some(phase) = phase
            && let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime)
        {
            return Err(self.refuse_elevation_request(program, cause.into()));
        }
        if let Err(outcome) = self.validate_commit_program(None, program.application_program()) {
            let mut inner = program.into_inner();
            let Some(binding) = inner.read_set.admission.take_elevation_request_binding() else {
                return Err(WorthQueryElevationRequestOutcome::Denied(
                    WorthQueryApplicationCommitDenial::elevation_request_program_mismatch(),
                ));
            };
            return Err(requested_outcome(outcome, binding));
        }
        Ok(program)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_elevation_approval_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        WorthQueryElevationApprovalOutcome,
    > {
        if let Some(phase) = phase
            && let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime)
        {
            return Err(self.refuse_elevation_approval(program, cause.into()));
        }
        if let Err(outcome) = self.validate_commit_program(None, program.application_program()) {
            let mut inner = program.into_inner();
            let Some(binding) = inner.read_set.admission.take_elevation_approval_binding() else {
                return Err(WorthQueryElevationApprovalOutcome::Indeterminate);
            };
            return Err(approved_outcome(outcome, binding));
        }
        Ok(program)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_elevation_close_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        WorthQueryElevationCloseOutcome,
    > {
        if let Some(phase) = phase
            && let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime)
        {
            return Err(self.refuse_elevation_close(program, cause.into()));
        }
        if let Err(outcome) = self.validate_commit_program(None, program.application_program()) {
            let mut inner = program.into_inner();
            let Some(binding) = inner.read_set.admission.take_elevation_close_binding() else {
                return Err(WorthQueryElevationCloseOutcome::Indeterminate);
            };
            return Err(closed_outcome(outcome, binding));
        }
        Ok(program)
    }

    pub(in crate::domain_computation::primary_graph) fn readmit_mandatory_review_installation<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: Option<&WorthQueryAdvancementPhase<'_>>,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        WorthQueryMandatoryReviewOutcome,
    > {
        if let Some(phase) = phase
            && let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime)
        {
            return Err(self.refuse_mandatory_review(program, cause.into()));
        }
        if let Err(outcome) = self.validate_commit_program(None, program.application_program()) {
            let mut inner = program.into_inner();
            let Some(binding) = inner.read_set.admission.take_mandatory_review_binding() else {
                return Err(WorthQueryMandatoryReviewOutcome::Indeterminate);
            };
            return Err(reviewed_outcome(outcome, binding));
        }
        Ok(program)
    }
}
