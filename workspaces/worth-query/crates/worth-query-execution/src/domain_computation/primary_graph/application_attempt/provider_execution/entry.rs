use crate::domain_computation::application_aftermath::ApplicationCommitCausality;
use crate::domain_computation::primary_graph::application_attempt::EntryCustodyScopedOps;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
};
use super::program_occurrence_gate::{
    require_occurrence_acts_through, require_selected_program_matches_occurrence,
    resolve_occurrence_program,
};
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;

pub(in crate::domain_computation) trait EntryScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn has_installed_application_program(&self) -> bool {
        self.program_support().is_some()
    }

    fn direct_operation_commit_denial<Operation: 'static>(
        &self,
    ) -> Option<WorthQueryApplicationCommitDenial> {
        if self
            .application_runtime()
            .operation_requires_workflow_authority::<Operation>()
        {
            Some(WorthQueryApplicationCommitDenial::workflow_authority_required())
        } else if self
            .application_runtime()
            .operation_requires_application_program::<Operation>()
        {
            Some(WorthQueryApplicationCommitDenial::program_lane_required::<
                Operation,
            >())
        } else {
            None
        }
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> EntryScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) trait ProviderExecutionEntryScopedOps<
    Schema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> where
    Schema: ApplicationSchema,
{
    fn require_occurrence_owns_output_source<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        presented: &WorthQueryPresentedProgram<'_>,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        let Some(support) = self.program_support().as_ref() else {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        let occurrence = resolve_occurrence_program(execution, support, program)
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

    fn require_occurrence_program_commit_binding<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        selected_program: Option<(
            &worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        )>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome>
    where
        Operation: 'static,
    {
        let Some(support) = self.program_support().as_ref() else {
            return Err(WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        let occurrence = resolve_occurrence_program(execution, support, program)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        require_selected_program_matches_occurrence(&occurrence, selected_program)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        require_occurrence_acts_through::<Operation>(&occurrence)
            .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
        Ok(())
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ProviderExecutionEntryScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(in crate::domain_computation::primary_graph) trait ProviderExecutionProviderExecutionEntryScopedOps<
    Schema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_conditional_operation<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_conditional_operation(execution, idempotency)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ProviderExecutionProviderExecutionEntryScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

mod application_commit;

mod output_source;
pub use application_commit::ApplicationCommitOps;
pub(in crate::domain_computation::primary_graph) use application_commit::ApplicationCommitScopedOps;
pub use output_source::OutputSourceOps;
