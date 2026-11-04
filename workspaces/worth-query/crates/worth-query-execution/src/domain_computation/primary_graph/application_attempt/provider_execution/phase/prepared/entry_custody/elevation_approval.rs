use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    approved_outcome, validate_elevation_approval_program,
};
use crate::domain_computation::primary_graph::WorthQueryElevationApprovalOutcome;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_approval_for_program(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        direct_denial: Option<WorthQueryApplicationCommitDenial>,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;
        let mut program = program.into_inner();
        if let Some(denial) = direct_denial {
            let Some(binding) = program.read_set.admission.take_elevation_approval_binding() else {
                return WorthQueryElevationApprovalOutcome::Indeterminate;
            };
            return WorthQueryElevationApprovalOutcome::Denied(denial, binding.into_requested());
        }
        if validate_elevation_approval_program(&program).is_err() {
            let binding = program
                .read_set
                .admission
                .take_elevation_approval_binding()
                .expect("typed approval programs retain their lifecycle binding");
            return WorthQueryElevationApprovalOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_approval_program_mismatch(),
                binding.into_requested(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_approval_binding()
            .map(WorthQueryElevationCommitCurrentness::approval);
        let Some(binding) = program.read_set.admission.take_elevation_approval_binding() else {
            return WorthQueryElevationApprovalOutcome::Indeterminate;
        };
        approved_outcome(
            CurrentCommitProgram {
                application,
                program,
                phase,
            }
            .compare_and_commit_application_inner_with_currentness(
                execution,
                idempotency,
                currentness,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
            binding,
        )
    }
}
