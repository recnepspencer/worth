use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    closed_outcome, validate_elevation_close_program,
};
use crate::domain_computation::primary_graph::WorthQueryElevationCloseOutcome;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_close_for_program(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        direct_denial: Option<WorthQueryApplicationCommitDenial>,
    ) -> WorthQueryElevationCloseOutcome
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
            let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
                return WorthQueryElevationCloseOutcome::Indeterminate;
            };
            return WorthQueryElevationCloseOutcome::Denied(denial, binding.into_approved());
        }
        if validate_elevation_close_program(&program).is_err() {
            let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
                return WorthQueryElevationCloseOutcome::Indeterminate;
            };
            return WorthQueryElevationCloseOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_close_program_mismatch(),
                binding.into_approved(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_close_binding()
            .map(WorthQueryElevationCommitCurrentness::close);
        let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
            return WorthQueryElevationCloseOutcome::Indeterminate;
        };
        closed_outcome(
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
