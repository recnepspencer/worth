//! Required-source and producer occurrence admission.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_required_output_source<
        Operation,
        Input,
        Scope,
    >(
        &self,
        presented: &WorthQueryPresentedProgram<'_>,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(outcome) = self.require_occurrence_owns_output_source(presented, &program) {
            return outcome;
        }
        self.compare_and_commit_application_with_output_observation(
            program,
            idempotency,
            true,
            allocation_policy,
        )
    }

    /// Requires that the program presented for this output source is the one
    /// active on the attempt's own occurrence. The program runtime already
    /// checked its typed root and source-binding inventory before entering here.
    fn require_occurrence_owns_output_source<Operation, Input, Scope>(
        &self,
        presented: &WorthQueryPresentedProgram<'_>,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        let Some(support) = self.program_support.as_ref() else {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        let occurrence = resolve_occurrence_program(support, program)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        if occurrence.rendering() != presented.rendering() {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::program_not_active_on_occurrence(
                    presented.identity(),
                    occurrence.entry().identity(),
                    occurrence.entry().revision(),
                ),
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_output_producer<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        selected_program: Option<(
            &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        )>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if !self
            .installed_conditionals
            .contains_operation::<Operation>()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::conditional_operation_not_installed::<Operation>(
                ),
            );
        }
        if let Err(outcome) =
            self.require_occurrence_program_commit_binding(&program, selected_program)
        {
            return outcome;
        }
        self.compare_and_commit_application_with_output_observation(
            program,
            idempotency,
            true,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }
}
