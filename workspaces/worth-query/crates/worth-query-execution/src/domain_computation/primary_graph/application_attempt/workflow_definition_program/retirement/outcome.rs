use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationUncommitted,
};
use super::PublishedWorkflowDefinitionRef;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

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
    definition: PublishedWorkflowDefinitionRef,
    receipt: WorthQueryApplicationCommitReceipt,
    replayed: bool,
}

impl PerformedWorkflowDefinitionRetirement {
    pub const fn definition(&self) -> &PublishedWorkflowDefinitionRef {
        &self.definition
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }
}

/// What executing a workflow definition retirement produced.
#[derive(Debug)]
pub enum WorkflowDefinitionRetirementOutcome {
    Retired(PerformedWorkflowDefinitionRetirement),
    /// The commit did not land. A landed commit, first or replayed, is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_workflow_definition_retirement<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
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
        let outcome = self.compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency.bind_workflow_definition(&workflow_intent_identity),
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let (receipt, replayed) = match outcome.landed() {
            Ok(landed) => landed,
            Err(uncommitted) => {
                return WorkflowDefinitionRetirementOutcome::Application(uncommitted)
            }
        };
        WorkflowDefinitionRetirementOutcome::Retired(PerformedWorkflowDefinitionRetirement {
            definition,
            receipt,
            replayed,
        })
    }
}
