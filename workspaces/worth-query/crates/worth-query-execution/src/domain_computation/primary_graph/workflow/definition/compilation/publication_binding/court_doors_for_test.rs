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

impl ColdCompiledWorkflowDefinition {
    #[doc(hidden)]
    pub fn new_for_test(
        lineage: EntityId,
        definition: EntityId,
        start_node: EntityId,
        nodes: Box<[CompiledWorkflowNode]>,
        connections: Box<[CompiledWorkflowConnection]>,
        global_actions: Box<
            [worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency],
        >,
        revisions: WorkflowDefinitionPublicationRevisions,
    ) -> Self {
        Self {
            lineage,
            definition,
            start_node,
            nodes,
            connections,
            global_actions,
            revisions,
        }
    }
}

impl WorkflowDefinitionPublicationRevisions {
    #[doc(hidden)]
    pub fn new_for_test(
        start: Option<VersionId>,
        nodes: Option<VersionId>,
        connections: Option<VersionId>,
    ) -> Self {
        Self {
            start,
            nodes,
            connections,
        }
    }
}
