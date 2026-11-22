use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_instance_program::publication::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationUncommitted};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_workflow_instance_start<Operation: 'static, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceStartOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime) {
            return WorkflowInstanceStartOutcome::Application(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
            );
        }
        let worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_instance_program::publication::PreparedWorkflowInstanceStartCommitParts {
            program,
            program_revision,
            definition,
            definition_content_identity,
            discovery_intent,
            instance_identity_locator,
            start_path,
            supersession,
        } = prepared.into_commit_parts(phase);
        let presented = match self
            .installed_program_support()
            .ok_or_else(
                super::super::WorthQueryApplicationCommitDenial::application_program_required,
            )
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowInstanceStartOutcome::Application(
                    WorthQueryApplicationUncommitted::Denied(denial),
                );
            }
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            execution,
            phase,
            &presented,
            program,
            idempotency.bind_workflow_instance(discovery_intent.intent_identity()),
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        match outcome.landed() {
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt }) => project(
                receipt,
                definition_content_identity,
                definition,
                program_revision,
                discovery_intent.lookup_identity(),
                instance_identity_locator,
                start_path,
                ),
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Recovered(receipt)) => WorkflowInstanceStartOutcome::RecoveredCommitted(receipt),
            Err(uncommitted) => match supersession {
                Some(supersession) => supersession
                    .outcome(uncommitted)
                    .unwrap_or_else(WorkflowInstanceStartOutcome::Application),
                None => WorkflowInstanceStartOutcome::Application(uncommitted),
            },
        }
    }
}
