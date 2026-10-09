use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_required_output_source<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

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
            phase,
            program,
            idempotency,
            true,
            allocation_policy,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_output_producer<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

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
            phase,
            program,
            idempotency,
            true,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_action<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        presented: &WorthQueryPresentedProgram,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if self
            .installed_conditionals
            .contains_operation::<Operation>()
            || !presented.acts_through_operation(std::any::TypeId::of::<Operation>())
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        let Some(support) = self.program_support.as_ref() else {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        };
        let occurrence = match resolve_occurrence_program(support, &program) {
            Ok(occurrence) => occurrence,
            Err(denial) => return WorthQueryApplicationCommitOutcome::Denied(denial),
        };
        if occurrence.rendering() != presented.rendering() {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::program_not_active_on_occurrence(
                    presented.identity(),
                    occurrence.entry().identity(),
                    occurrence.entry().revision(),
                ),
            );
        }
        self.compare_and_commit_application_with_causality(
            phase,
            program,
            idempotency,
            false,
            causality,
            allocation_policy,
        )
    }
}
