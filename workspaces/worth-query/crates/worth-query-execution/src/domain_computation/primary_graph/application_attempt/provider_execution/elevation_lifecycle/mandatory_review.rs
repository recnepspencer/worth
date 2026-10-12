use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_mandatory_review<Operation, Input, Scope>(
        &self,
        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let request = program.request_scope().clone();
        let mut candidate = Some(program);
        self.with_application_advancement(&request, |active_phase| {
            let execution = active_phase
                .execution_request_for(&self.product_runtime)
                .map_err(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from,
                )?;
            Ok(self.compare_and_commit_mandatory_review_for_program(
                &execution,
                &active_phase,
                candidate.take().expect("candidate admitted once"),
                idempotency,
                self.direct_operation_commit_denial::<Operation>(),
            ))
        })
        .and_then(std::convert::identity)
        .unwrap_or_else(|denial| match candidate.take() {
            Some(candidate) => self.refuse_mandatory_review(candidate, denial),
            None => WorthQueryMandatoryReviewOutcome::Indeterminate,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_mandatory_review_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryMandatoryReviewProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        direct_denial: Option<WorthQueryApplicationCommitDenial>,
    ) -> WorthQueryMandatoryReviewOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
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
            self.compare_and_commit_application_inner(
                execution,
                phase,
                program,
                idempotency,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
            binding,
        )
    }
}
