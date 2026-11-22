pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_definition_program::publication::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationUncommitted};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Opaque, live application-attempt publication program.
///

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_workflow_definition_publication<Operation: 'static, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowDefinitionPublicationOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let (
            program,
            program_revision,
            content_identity,
            content_identity_locator,
            workflow_intent_identity,
        ) = prepared.into_parts();
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => {
                return WorkflowDefinitionPublicationOutcome::Application(
                    outcome.landed().expect_err("a refused entry cannot commit"),
                )
            }
        };
        let presented = match self
            .installed_program_support()
            .ok_or_else(
                super::super::WorthQueryApplicationCommitDenial::application_program_required,
            )
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowDefinitionPublicationOutcome::Application(
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
        match outcome.landed() {
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt }) => project_published(
                receipt,
                content_identity,
                &content_identity_locator,
                ),
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Recovered(receipt)) => WorkflowDefinitionPublicationOutcome::RecoveredCommitted(receipt),
            Err(uncommitted) => WorkflowDefinitionPublicationOutcome::Application(uncommitted),
        }
    }
}
