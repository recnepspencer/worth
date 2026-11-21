use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationUncommitted,
};
use super::super::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_instance_program::cancellation::PreparedWorkflowInstanceCancellation;

/// A canceled instance and the effects it performed before it ended, which
/// cancellation never reverses.
#[derive(Debug)]
pub struct PerformedWorkflowInstanceCancellation {
    instance: PublishedWorkflowInstanceRef,
    receipt: WorthQueryApplicationCommitReceipt,
    performed: Vec<String>,
}

impl PerformedWorkflowInstanceCancellation {
    pub const fn instance(&self) -> &PublishedWorkflowInstanceRef {
        &self.instance
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    /// The path of each node whose effect the instance, or the source it was
    /// migrated from, performed. Each remains performed.
    pub fn performed_node_paths(&self) -> &[String] {
        &self.performed
    }
}

/// What executing a workflow instance cancellation produced. `ProjectionDenied` means the
/// commit landed but its receipt does not record this cancellation on the instance.
#[derive(Debug)]
pub enum WorkflowInstanceCancellationOutcome {
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    Cancelled(PerformedWorkflowInstanceCancellation),
    /// The commit did not land. A fresh landed commit is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
    ProjectionDenied(WorthQueryApplicationCommitReceipt),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_workflow_instance_cancellation<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceCancellationOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime) {
            return WorkflowInstanceCancellationOutcome::Application(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
            );
        }
        let PreparedWorkflowInstanceCancellation {
            program,
            program_revision,
            instance,
            cancellation_identity,
            intent_identity,
            cancellation_identity_locator,
            performed,
        } = prepared;
        let presented = match self
            .installed_program_support()
            .ok_or_else(WorthQueryApplicationCommitDenial::application_program_required)
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowInstanceCancellationOutcome::Application(
                    WorthQueryApplicationUncommitted::Denied(denial),
                );
            }
        };
        let receipt = match self
            .compare_and_commit_application_for_program_action(execution,
                phase,
                &presented,
                program,
                idempotency.bind_workflow_instance(&intent_identity),
                crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .landed()
        {
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt }) => receipt,
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Recovered(receipt)) => return WorkflowInstanceCancellationOutcome::RecoveredCommitted(receipt),
            Err(uncommitted) => {
                return WorkflowInstanceCancellationOutcome::Application(uncommitted)
            }
        };
        // The receipt must record this cancellation on this instance.
        let expected = AspectValue::String(InternedString::Raw(cancellation_identity));
        if receipt
            .committed_changes()
            .committed_field_values(instance.entity_id(), &[&cancellation_identity_locator])
            != Some(vec![expected])
        {
            return WorkflowInstanceCancellationOutcome::ProjectionDenied(receipt);
        }
        WorkflowInstanceCancellationOutcome::Cancelled(PerformedWorkflowInstanceCancellation {
            instance,
            receipt,
            performed,
        })
    }
}
