//! Sealed test doors over this module's private members.
use super::*;

impl CompiledWorkflowSemanticConnection {
    #[doc(hidden)]
    pub fn kind_for_test(&self) -> &Arc<CompiledWorkflowConnectionKind> {
        &self.kind
    }
}

impl CompiledWorkflowSemanticConnection {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> &usize {
        &self.source
    }
}

impl CompiledWorkflowSemanticConnection {
    #[doc(hidden)]
    pub fn target_for_test(&self) -> &usize {
        &self.target
    }
}

impl CompiledWorkflowDefinition {
    #[doc(hidden)]
    pub fn definition_for_test(&self) -> EntityId {
        Self::definition(self)
    }
}

impl CompiledWorkflowNode {
    #[doc(hidden)]
    pub fn entity_for_test(&self) -> EntityId {
        Self::entity(self)
    }
}

impl CompiledWorkflowDefinition {
    #[doc(hidden)]
    pub fn lineage_for_test(&self) -> EntityId {
        Self::lineage(self)
    }
}

impl CompiledWorkflowDefinition {
    #[doc(hidden)]
    pub fn new_for_test(
        publication: Arc<CompiledWorkflowPublicationPlan>,
        content_identity: ApplicationWorkflowDefinitionContentIdentity,
        program_revision: ApplicationProgramRevision,
        resume: Option<usize>,
    ) -> Self {
        Self {
            publication,
            content_identity,
            program_revision,
            resume,
        }
    }
}

impl CompiledWorkflowNodeMeaning {
    #[doc(hidden)]
    pub fn new_for_test(path: String, kind: CompiledWorkflowNodeKind) -> Self {
        Self { path, kind }
    }
}

impl CompiledWorkflowPublicationPlan {
    #[doc(hidden)]
    pub fn new_for_test(
        lineage: EntityId,
        definition: EntityId,
        start: usize,
        node_ordinals: Arc<[(EntityId, usize)]>,
        nodes: Box<[CompiledWorkflowNode]>,
        connections: Box<[CompiledWorkflowConnection]>,
        global_actions: Arc<
            [worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency],
        >,
        dispatch: Arc<CompiledWorkflowDispatch>,
    ) -> Self {
        Self {
            lineage,
            definition,
            start,
            node_ordinals,
            nodes,
            connections,
            global_actions,
            dispatch,
        }
    }
}

impl CompiledWorkflowConnection {
    #[doc(hidden)]
    pub fn new_for_test(
        entity: EntityId,
        source: EntityId,
        target: EntityId,
        kind: Arc<CompiledWorkflowConnectionKind>,
    ) -> Self {
        Self {
            entity,
            source,
            target,
            kind,
        }
    }
}

impl CompiledWorkflowNode {
    #[doc(hidden)]
    pub fn new_for_test(entity: EntityId, meaning: Arc<CompiledWorkflowNodeMeaning>) -> Self {
        Self { entity, meaning }
    }
}

impl CompiledWorkflowSemanticConnection {
    #[doc(hidden)]
    pub fn new_for_test(
        source: usize,
        target: usize,
        kind: Arc<CompiledWorkflowConnectionKind>,
    ) -> Self {
        Self {
            source,
            target,
            kind,
        }
    }
}
