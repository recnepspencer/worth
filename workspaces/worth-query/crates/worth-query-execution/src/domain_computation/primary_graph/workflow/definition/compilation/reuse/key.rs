use worth_query_declaration::facade::application_program::{
    ApplicationProgramRevision, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_relational::facade::identity::EntityId;

use super::{
    WorkflowDefinitionCompilationReuse, DEFAULT_WORKFLOW_COMPILATION_RETAINED_BYTE_BUDGET,
};

impl Default for WorkflowDefinitionCompilationReuse {
    fn default() -> Self {
        Self::new(DEFAULT_WORKFLOW_COMPILATION_RETAINED_BYTE_BUDGET)
    }
}

#[cfg(test)]
impl WorkflowDefinitionCompilationReuse {
    pub(super) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) enum WorkflowDefinitionCompilationReuseDenial
{
    SemanticCollision,
    ByteBudgetExceeded,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) struct WorkflowDefinitionSemanticReuseKey
{
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) content_identity:
        String,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) program_revision:
        String,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) vocabulary_identity:
        String,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) support_identity:
        [u8; 32],
}

impl WorkflowDefinitionSemanticReuseKey {
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) fn new(
        content_identity: &ApplicationWorkflowDefinitionContentIdentity,
        program_revision: &ApplicationProgramRevision,
        vocabulary_identity: &str,
        support_identity: &[u8; 32],
    ) -> Self {
        Self {
            content_identity: content_identity.to_string(),
            program_revision: program_revision.to_string(),
            vocabulary_identity: vocabulary_identity.to_owned(),
            support_identity: *support_identity,
        }
    }

    pub(super) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.content_identity.len())
            .saturating_add(self.program_revision.len())
            .saturating_add(self.vocabulary_identity.len())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) struct WorkflowDefinitionPublicationReuseKey
{
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) branch_occurrence:
        u64,
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) definition:
        EntityId,
}

impl WorkflowDefinitionPublicationReuseKey {
    pub(in crate::domain_computation::primary_graph::workflow::definition::compilation) const fn new(
        branch: crate::basis::WorthQueryProductBranch,
        definition: EntityId,
    ) -> Self {
        Self {
            branch_occurrence: branch.occurrence_ordinal(),
            definition,
        }
    }
}
