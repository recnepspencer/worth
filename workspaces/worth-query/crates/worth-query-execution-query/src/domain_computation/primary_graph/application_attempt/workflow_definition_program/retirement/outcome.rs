use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationUncommitted,
};
use crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_definition_program::retirement::outcome::{
    PerformedWorkflowDefinitionRetirement, PreparedWorkflowDefinitionRetirement,
    WorkflowDefinitionRetirementOutcome,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_workflow_definition_retirement<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionRetirementOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let PreparedWorkflowDefinitionRetirement {
            program,
            program_revision,
            definition,
            workflow_intent_identity,
        } = prepared;
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => {
                return WorkflowDefinitionRetirementOutcome::Application(
                    outcome.landed().expect_err("a refused entry cannot commit"),
                )
            }
        };
        let presented = match self
            .installed_program_support()
            .ok_or_else(WorthQueryApplicationCommitDenial::application_program_required)
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowDefinitionRetirementOutcome::Application(
                    WorthQueryApplicationUncommitted::Denied(denial),
                );
            }
        };
        let outcome = entry.compare_and_commit_application_for_program_action(
            execution,
            &presented,
            idempotency.bind_workflow_definition(&workflow_intent_identity),
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let receipt = match outcome.landed() {
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt }) => receipt,
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Recovered(receipt)) => return WorkflowDefinitionRetirementOutcome::RecoveredCommitted(receipt),
            Err(uncommitted) => {
                return WorkflowDefinitionRetirementOutcome::Application(uncommitted)
            }
        };
        WorkflowDefinitionRetirementOutcome::Retired(PerformedWorkflowDefinitionRetirement {
            definition,
            receipt,
        })
    }
}
