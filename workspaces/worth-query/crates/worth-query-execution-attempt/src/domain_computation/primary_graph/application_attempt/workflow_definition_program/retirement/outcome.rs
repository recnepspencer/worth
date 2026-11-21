use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::super::super::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationUncommitted,
};
use super::PublishedWorkflowDefinitionRef;
/// Opaque, live retirement program for one exact published definition.
///
/// Callers cannot name the lineage, the current-pointer relation, or the
/// effects; the owner observes them and rechecks currentness at commit.
#[must_use = "prepared workflow definition retirement owns a live application attempt"]
pub struct PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) definition: PublishedWorkflowDefinitionRef,
    pub(super) workflow_intent_identity: [u8; 32],
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>
{
    pub const fn definition(&self) -> &PublishedWorkflowDefinitionRef {
        &self.definition
    }
}

/// One definition proven no longer current by an application commit.
#[derive(Debug)]
pub struct PerformedWorkflowDefinitionRetirement {
    pub(super) definition: PublishedWorkflowDefinitionRef,
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
}

impl<Schema, Operation, Input, Scope>
    crate::ports::WorkflowDefinitionRetirementPort<Schema, Operation, Input, Scope>
    for PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>
{
    fn into_retirement_commit_parts(
        self,
    ) -> (
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        ApplicationProgramRevision,
        PublishedWorkflowDefinitionRef,
        [u8; 32],
    ) {
        (
            self.program,
            self.program_revision,
            self.definition,
            self.workflow_intent_identity,
        )
    }

    fn complete_retirement(
        definition: PublishedWorkflowDefinitionRef,
        receipt: WorthQueryApplicationCommitReceipt,
    ) -> PerformedWorkflowDefinitionRetirement {
        PerformedWorkflowDefinitionRetirement {
            definition,
            receipt,
        }
    }
}

impl PerformedWorkflowDefinitionRetirement {
    pub const fn definition(&self) -> &PublishedWorkflowDefinitionRef {
        &self.definition
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

/// What executing a workflow definition retirement produced.
#[derive(Debug)]
pub enum WorkflowDefinitionRetirementOutcome {
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    Retired(PerformedWorkflowDefinitionRetirement),
    /// The commit did not land. A fresh landed commit is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>
{
    /// Controls retained as facts; each host call obtains fresh execution admission.
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.request_scope()
    }
}
