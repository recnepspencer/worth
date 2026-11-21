use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationUncommitted,
};
use super::PublishedWorkflowDefinitionRef;
/// Opaque, live retirement program for one exact published definition.
///
/// Callers cannot name the lineage, the current-pointer relation, or the
/// effects; the owner observes them and rechecks currentness at commit.
///
/// ```compile_fail,E0451
/// use worth_query_execution::publication_boundary::workflow_definition_retirement::PreparedWorkflowDefinitionRetirement;
///
/// fn cannot_forge<Schema, Operation, Input, Scope>()
///     -> PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope>
/// {
///     PreparedWorkflowDefinitionRetirement {
///         program: unsafe { std::mem::zeroed() },
///         program_revision: unsafe { std::mem::zeroed() },
///         definition: unsafe { std::mem::zeroed() },
///         workflow_intent_identity: [0; 32],
///     }
/// }
/// ```
#[must_use = "prepared workflow definition retirement owns a live application attempt"]
pub struct PreparedWorkflowDefinitionRetirement<Schema, Operation, Input, Scope> {
    pub program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub program_revision: ApplicationProgramRevision,
    pub definition: PublishedWorkflowDefinitionRef,
    pub workflow_intent_identity: [u8; 32],
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
    pub definition: PublishedWorkflowDefinitionRef,
    pub receipt: WorthQueryApplicationCommitReceipt,
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
