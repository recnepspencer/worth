use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Opaque, live application-attempt publication program.
///
/// Construction consumes an admitted projected read set and an owner-bound
/// definition contract. Callers cannot provide lineage IDs, current-pointer
/// relation IDs, effects, or branch affinity.
///
/// ```compile_fail
/// use worth_query_execution::facade::workflow_definition_publication::PreparedWorkflowDefinitionPublication;
///
/// fn cannot_forge<Schema, Operation, Input, Scope>()
///     -> PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>
/// {
///     PreparedWorkflowDefinitionPublication {
///         program: unsafe { std::mem::zeroed() },
///         content_identity: unsafe { std::mem::zeroed() },
///     }
/// }
/// ```
#[must_use = "prepared workflow definition publication owns a live application attempt"]
pub struct PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope> {
    pub(super) program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    pub(super) program_revision: ApplicationProgramRevision,
    pub(super) content_identity: ApplicationWorkflowDefinitionContentIdentity,
    pub(super) content_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub(super) workflow_intent_identity: [u8; 32],
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowDefinitionPublication<Schema, Operation, Input, Scope>
{
    pub fn content_identity(&self) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }

    fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        ApplicationProgramRevision,
        ApplicationWorkflowDefinitionContentIdentity,
        worth_foundational::facade::AspectFieldLocator,
        [u8; 32],
    ) {
        (
            self.program,
            self.program_revision,
            self.content_identity,
            self.content_identity_locator,
            self.workflow_intent_identity,
        )
    }
}

/// Opaque identity of one definition revision proven by an application commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedWorkflowDefinitionRef {
    branch: crate::basis::WorthQueryProductBranch,
    entity_id: worth_relational::facade::identity::EntityId,
    content_identity: ApplicationWorkflowDefinitionContentIdentity,
}

/// The exact branch-current state an author observed before requesting a
/// definition publication. A revision never overwrites a newer predecessor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowDefinitionExpectedPredecessor {
    Absent,
    Published(PublishedWorkflowDefinitionRef),
}

impl WorkflowDefinitionExpectedPredecessor {
    pub(super) fn belongs_to_branch(&self, branch: crate::basis::WorthQueryProductBranch) -> bool {
        match self {
            Self::Absent => true,
            Self::Published(predecessor) => predecessor.branch == branch,
        }
    }

    pub(super) fn definition_entity(&self) -> Option<worth_relational::facade::identity::EntityId> {
        match self {
            Self::Absent => None,
            Self::Published(predecessor) => Some(predecessor.entity_id),
        }
    }
}

impl PublishedWorkflowDefinitionRef {
    pub(in crate::domain_computation::primary_graph) fn retained(
        branch: crate::basis::WorthQueryProductBranch,
        entity_id: worth_relational::facade::identity::EntityId,
        content_identity: ApplicationWorkflowDefinitionContentIdentity,
    ) -> Self {
        Self {
            branch,
            entity_id,
            content_identity,
        }
    }

    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub const fn entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.entity_id
    }

    pub const fn content_identity(&self) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }
}

#[derive(Debug)]
pub struct PerformedWorkflowDefinitionPublication {
    definition: PublishedWorkflowDefinitionRef,
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    replayed: bool,
}

impl PerformedWorkflowDefinitionPublication {
    pub const fn definition(&self) -> &PublishedWorkflowDefinitionRef {
        &self.definition
    }

    pub const fn receipt(&self) -> &super::super::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn replayed(&self) -> bool {
        self.replayed
    }
}

#[derive(Debug)]
pub enum WorkflowDefinitionPublicationOutcome {
    Published(PerformedWorkflowDefinitionPublication),
    Application(WorthQueryApplicationCommitOutcome),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_workflow_definition_publication<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
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
        let Some(presented) = self
            .installed_program_support()
            .and_then(|support| support.present(&program_revision))
        else {
            return WorkflowDefinitionPublicationOutcome::Application(
                WorthQueryApplicationCommitOutcome::Denied(
                    super::super::WorthQueryApplicationCommitDenial::application_program_required(),
                ),
            );
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            &presented,
            program,
            idempotency.bind_workflow_definition(&workflow_intent_identity),
        );
        match outcome {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => {
                project_published(receipt, content_identity, &content_identity_locator, false)
            }
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
                project_published(receipt, content_identity, &content_identity_locator, true)
            }
            other => WorkflowDefinitionPublicationOutcome::Application(other),
        }
    }
}

fn project_published(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    content_identity: ApplicationWorkflowDefinitionContentIdentity,
    locator: &worth_foundational::facade::AspectFieldLocator,
    replayed: bool,
) -> WorkflowDefinitionPublicationOutcome {
    let expected = AspectValue::String(InternedString::Raw(content_identity.to_string()));
    let candidates = receipt
        .committed_changes()
        .created_entities_with_field_value(locator, &expected);
    let [entity_id] = candidates.as_slice() else {
        return WorkflowDefinitionPublicationOutcome::ProjectionDenied(receipt);
    };
    let definition = PublishedWorkflowDefinitionRef {
        branch: receipt.product_branch(),
        entity_id: *entity_id,
        content_identity,
    };
    WorkflowDefinitionPublicationOutcome::Published(PerformedWorkflowDefinitionPublication {
        definition,
        receipt,
        replayed,
    })
}
