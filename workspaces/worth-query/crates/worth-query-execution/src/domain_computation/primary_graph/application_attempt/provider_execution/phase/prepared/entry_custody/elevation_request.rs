use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    requested_outcome, validate_elevation_request_program,
};
use crate::domain_computation::primary_graph::WorthQueryElevationRequestOutcome;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_request_for_program(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;
        let mut program = program.into_inner();
        if validate_elevation_request_program(&program).is_err() {
            return WorthQueryElevationRequestOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_request_program_mismatch(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_request_binding()
            .map(WorthQueryElevationCommitCurrentness::request);
        let Some(binding) = program.read_set.admission.take_elevation_request_binding() else {
            return WorthQueryElevationRequestOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_request_program_mismatch(),
            );
        };
        requested_outcome(
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
