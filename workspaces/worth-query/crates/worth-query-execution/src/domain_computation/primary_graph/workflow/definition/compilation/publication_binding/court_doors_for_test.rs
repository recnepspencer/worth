//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowDefinitionPublicationBinding {
    #[doc(hidden)]
    pub fn bind_for_test(
        &self,
        semantic: Arc<CompiledWorkflowSemanticPlan>,
        content_identity: worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity,
        program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    ) -> Result<CompiledWorkflowDefinition, WorkflowDefinitionPublicationBindingDenial> {
        Self::bind(self, semantic, content_identity, program_revision)
    }
}
