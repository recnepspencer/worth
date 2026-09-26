use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding,
};
use super::super::PublishedWorkflowInstanceRef;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[must_use = "prepared workflow instance cancellation owns a live application attempt"]
pub struct PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) instance: PublishedWorkflowInstanceRef,
    pub(super) cancellation_identity: String,
    pub(super) intent_identity: [u8; 32],
    pub(super) cancellation_identity_locator: AspectFieldLocator,
    pub(super) performed: Vec<String>,
}

/// A cancelled instance and the effects it performed before it ended, which
/// cancellation never reverses.
#[derive(Debug)]
pub struct PerformedWorkflowInstanceCancellation {
    instance: PublishedWorkflowInstanceRef,
    receipt: WorthQueryApplicationCommitReceipt,
    replayed: bool,
    performed: Vec<String>,
}

impl PerformedWorkflowInstanceCancellation {
    pub const fn instance(&self) -> &PublishedWorkflowInstanceRef {
        &self.instance
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }

    /// The path of each node whose effect the instance, or the source it was
    /// migrated from, performed. Each remains performed.
    pub fn performed_node_paths(&self) -> &[String] {
        &self.performed
    }
}

#[derive(Debug)]
pub enum WorkflowInstanceCancellationOutcome {
    Cancelled(PerformedWorkflowInstanceCancellation),
    Application(WorthQueryApplicationCommitOutcome),
    ProjectionDenied(WorthQueryApplicationCommitReceipt),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_workflow_instance_cancellation<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
        prepared: PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceCancellationOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
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
                    WorthQueryApplicationCommitOutcome::Denied(denial),
                );
            }
        };
        let (receipt, replayed) = match self.compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency.bind_workflow_instance(&intent_identity),
        ) {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => (receipt, false),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => (receipt, true),
            other => return WorkflowInstanceCancellationOutcome::Application(other),
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
            replayed,
            performed,
        })
    }
}
