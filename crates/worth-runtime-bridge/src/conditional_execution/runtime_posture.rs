use super::BridgeOwnedSignalRuntime;

impl BridgeOwnedSignalRuntime {
    pub fn baseline_semantic_dependency_count(&self) -> usize {
        self.baseline_semantic_dependency_count
    }

    pub fn active_semantic_dependency_count(&self) -> usize {
        self.bridge
            .semantic_dependency_registry
            .authoritative_count()
    }

    pub fn revoke_conditional_liveness(&self) {
        for lowering in self
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
        {
            lowering.lease.revoke_liveness();
        }
    }
}
