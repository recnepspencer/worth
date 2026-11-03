use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    reviewed_outcome, validate_mandatory_review_program,
};
use crate::domain_computation::primary_graph::WorthQueryMandatoryReviewOutcome;
impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_mandatory_review_for_program(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        direct_denial: Option<WorthQueryApplicationCommitDenial>,
    ) -> WorthQueryMandatoryReviewOutcome
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
            let Some(binding) = program.read_set.admission.take_mandatory_review_binding() else {
                return WorthQueryMandatoryReviewOutcome::Indeterminate;
            };
            return WorthQueryMandatoryReviewOutcome::Denied(denial, binding.into_mandatory());
        }
        if validate_mandatory_review_program(&program).is_err() {
            let Some(binding) = program.read_set.admission.take_mandatory_review_binding() else {
                return WorthQueryMandatoryReviewOutcome::Indeterminate;
            };
            return WorthQueryMandatoryReviewOutcome::Denied(
                WorthQueryApplicationCommitDenial::mandatory_review_program_mismatch(),
                binding.into_mandatory(),
            );
        }
        let Some(binding) = program.read_set.admission.take_mandatory_review_binding() else {
            return WorthQueryMandatoryReviewOutcome::Indeterminate;
        };
        reviewed_outcome(
            CurrentCommitProgram {
                application,
                program,
                phase,
            }
            .compare_and_commit_application_inner(
                execution,
                idempotency,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
            binding,
        )
    }
}
