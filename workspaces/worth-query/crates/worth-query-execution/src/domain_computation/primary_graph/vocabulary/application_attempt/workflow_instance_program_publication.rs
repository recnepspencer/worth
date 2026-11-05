//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow_instance_program::publication`: the items below and every impl they own.
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity;

/// A reference to a started workflow instance: its branch, the branch it was started on,
/// its entity, the definition content identity and program revision it runs, and its start
/// node path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedWorkflowInstanceRef {
    pub(in crate::domain_computation::primary_graph) branch: crate::basis::WorthQueryProductBranch,
    /// The branch the instance was started on, whose occurrence its facts
    /// record. Only a fork's copy is read on another branch.
    pub(in crate::domain_computation::primary_graph) started_on:
        crate::basis::WorthQueryProductBranch,
    pub(in crate::domain_computation::primary_graph) entity_id:
        worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation::primary_graph) definition_content_identity:
        ApplicationWorkflowDefinitionContentIdentity,
    pub(in crate::domain_computation::primary_graph) definition:
        worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation::primary_graph) program_revision: ApplicationProgramRevision,
    pub(in crate::domain_computation::primary_graph) start_path: String,
}

impl PublishedWorkflowInstanceRef {
    /// Native governed reader validates occurrence/definition/current admission separately.
    /// The original native intent supplies the immutable start revision.
    pub(in crate::domain_computation::primary_graph) fn observed_current(
        branch: crate::basis::WorthQueryProductBranch,
        entity_id: worth_relational::facade::identity::EntityId,
        definition: worth_relational::facade::identity::EntityId,
        definition_content_identity: ApplicationWorkflowDefinitionContentIdentity,
        program_revision: ApplicationProgramRevision,
        start_path: String,
    ) -> Self {
        Self {
            branch,
            started_on: branch,
            entity_id,
            definition,
            definition_content_identity,
            program_revision,
            start_path,
        }
    }

    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub const fn entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.entity_id
    }

    /// The occurrence the instance's own facts record.
    pub(in crate::domain_computation::primary_graph) const fn started_occurrence(&self) -> u64 {
        self.started_on.occurrence_ordinal()
    }

    /// The same instance as `fork` copied it: read on the fork, still
    /// recording the branch it was started on.
    pub(in crate::domain_computation::primary_graph) fn copied_onto(
        &self,
        fork: crate::basis::WorthQueryProductBranch,
    ) -> Self {
        Self {
            branch: fork,
            ..self.clone()
        }
    }

    pub const fn definition_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.definition
    }

    /// The revision the instance started under. Program adoption may carry
    /// the instance to a later revision; advancing reads the revision from
    /// instance truth and never from this reference.
    pub const fn program_revision(&self) -> &ApplicationProgramRevision {
        &self.program_revision
    }

    pub const fn definition_content_identity(
        &self,
    ) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.definition_content_identity
    }

    pub fn start_node_path(&self) -> &str {
        &self.start_path
    }
}
