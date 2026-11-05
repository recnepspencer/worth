//! Sealed test doors over this module's private members.
use super::*;

impl CompiledWorkflowDispatch {
    #[doc(hidden)]
    pub fn build_for_test(
        node_count: usize,
        connections: &[CompiledWorkflowSemanticConnection],
    ) -> Self {
        Self::build(node_count, connections)
    }
}
