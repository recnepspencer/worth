use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationUncommitted,
};
use super::super::PublishedWorkflowInstanceRef;
#[must_use = "prepared workflow instance cancellation owns a live application attempt"]
pub struct PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope> {
    pub program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub program_revision: ApplicationProgramRevision,
    pub instance: PublishedWorkflowInstanceRef,
    pub cancellation_identity: String,
    pub intent_identity: [u8; 32],
    pub cancellation_identity_locator: AspectFieldLocator,
    pub performed: Vec<String>,
}

/// A canceled instance and the effects it performed before it ended, which
/// cancellation never reverses.
#[derive(Debug)]
pub struct PerformedWorkflowInstanceCancellation {
    pub instance: PublishedWorkflowInstanceRef,
    pub receipt: WorthQueryApplicationCommitReceipt,
    pub performed: Vec<String>,
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


impl<Schema, Operation, Input, Scope>
    PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>
{
    /// Controls retained as facts; each host call obtains fresh execution admission.
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.request_scope()
    }
}
