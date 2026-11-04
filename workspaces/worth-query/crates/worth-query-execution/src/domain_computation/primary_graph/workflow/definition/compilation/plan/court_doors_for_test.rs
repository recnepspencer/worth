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
