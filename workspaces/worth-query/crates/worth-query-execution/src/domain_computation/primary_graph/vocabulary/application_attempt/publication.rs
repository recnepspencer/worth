//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow_definition_program::publication`: the items below and every impl they own.
use worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity;

/// Opaque identity of one definition revision proven by an application commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedWorkflowDefinitionRef {
    branch: crate::basis::WorthQueryProductBranch,
    entity_id: worth_relational::facade::identity::EntityId,
    content_identity: ApplicationWorkflowDefinitionContentIdentity,
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

    /// The same definition as `branch` holds it. A fork copies definitions
    /// under their own identity, so this is the canonical name for the copy
    /// in requests on the fork, including fork continuation targets.
    /// It grants nothing: `branch`'s own truth decides every start, successor
    /// and retirement, and a definition `branch` does not hold current, such
    /// as one on a branch it was never forked from, is refused or stale there.
    #[must_use]
    pub fn held_on(&self, branch: crate::basis::WorthQueryProductBranch) -> Self {
        Self {
            branch,
            ..self.clone()
        }
    }

    pub const fn entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.entity_id
    }

    pub const fn content_identity(&self) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.content_identity
    }
}
