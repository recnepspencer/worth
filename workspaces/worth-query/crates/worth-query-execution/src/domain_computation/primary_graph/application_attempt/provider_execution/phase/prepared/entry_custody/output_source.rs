//! Commit work requires the validation owner's exact program custody.
use super::super::super::super::program_occurrence_gate::resolve_occurrence_program;
use super::*;
use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
use crate::domain_computation::primary_graph::application_attempt::provider_execution::ProviderExecutionEntryScopedOps;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
use crate::equality_type_id::EqualityTypeId as TypeId;

impl<Schema: ApplicationSchema, Operation, Input, Scope>
    CurrentCommitProgram<
        '_,
        '_,
        '_,
        Schema,
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    >
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_required_output_source(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        presented: &WorthQueryPresentedProgram<'_>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;

        if let Err(outcome) =
            application.require_occurrence_owns_output_source(execution, presented, &program)
        {
            return outcome;
        }
        let Some(support) = application.program_support().as_ref() else {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        };
        let program = match program.retain_original_program(support, presented.revision()) {
            Ok(program) => program,
            Err(denial) => return WorthQueryApplicationCommitOutcome::Denied(denial),
        };
        Self {
            application,
            program,
            phase,
        }
        .compare_and_commit_application_with_causality(
            execution,
            idempotency,
            true,
            causality,
            None,
            allocation_policy,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_output_producer(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        idempotency: WorthQueryApplicationIdempotencyBinding,
        selected_program: Option<(
            &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        )>,
        history: Option<HistoryProducerInvocation<'_>>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;

        if !application
            .installed_conditionals()
            .contains_operation::<Operation>()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::conditional_operation_not_installed::<Operation>(
                ),
            );
        }
        if let Err(outcome) = application.require_occurrence_program_commit_binding(
            execution,
            &program,
            selected_program,
        ) {
            return outcome;
        }
        Self {
            application,
            program,
            phase,
        }
        .compare_and_commit_application_with_output_observation(
            execution,
            idempotency,
            true,
            history,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_application_for_program_action(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        presented: &WorthQueryPresentedProgram,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        causality: ApplicationCommitCausality,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let Self {
            application,
            program,
            phase,
        } = self;

        if application
            .installed_conditionals()
            .contains_operation::<Operation>()
            || !presented.acts_through_operation(TypeId::of::<Operation>())
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        let Some(support) = application.program_support().as_ref() else {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        };
        let occurrence = match resolve_occurrence_program(execution, support, &program) {
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
        let program = match program.retain_original_program(support, occurrence.entry().revision())
        {
            Ok(program) => program,
            Err(denial) => return WorthQueryApplicationCommitOutcome::Denied(denial),
        };
        Self {
            application,
            program,
            phase,
        }
        .compare_and_commit_application_with_causality(
            execution,
            idempotency,
            false,
            causality,
            None,
            allocation_policy,
        )
    }
}
