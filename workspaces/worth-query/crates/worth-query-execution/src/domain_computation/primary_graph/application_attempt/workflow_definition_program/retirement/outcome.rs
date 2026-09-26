use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding,
};
use super::PublishedWorkflowDefinitionRef;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Opaque, live retirement program for one exact published definition.
///
/// Callers cannot name the lineage, the current-pointer relation, or the
/// effects; the owner observes them and rechecks currentness at commit.
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::workflow_definition_retirement::PreparedWorkflowDefinitionRetirement;
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

#[derive(Debug)]
pub enum WorkflowDefinitionRetirementOutcome {
    Retired(PerformedWorkflowDefinitionRetirement),
    Application(WorthQueryApplicationCommitOutcome),
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
        let Some(presented) = self
            .installed_program_support()
            .and_then(|support| support.present(&program_revision))
        else {
            return WorkflowDefinitionRetirementOutcome::Application(
                WorthQueryApplicationCommitOutcome::Denied(
                    WorthQueryApplicationCommitDenial::application_program_required(),
                ),
            );
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency.bind_workflow_definition(&workflow_intent_identity),
        );
        let (receipt, replayed) = match outcome {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => (receipt, false),
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => (receipt, true),
            other => return WorkflowDefinitionRetirementOutcome::Application(other),
        };
        WorkflowDefinitionRetirementOutcome::Retired(PerformedWorkflowDefinitionRetirement {
            definition,
            receipt,
            replayed,
        })
    }
}
