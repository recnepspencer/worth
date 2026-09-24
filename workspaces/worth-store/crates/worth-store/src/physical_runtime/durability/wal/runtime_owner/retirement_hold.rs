use super::PhysicalWalRuntimeOwner;

impl PhysicalWalRuntimeOwner {
    pub(in crate::physical_runtime) fn install_retirement_holds(
        &self,
        holds: Vec<(
            crate::physical_runtime::durability::RetiredArtifact,
            u64,
            u64,
        )>,
    ) {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unresolved_retirement_spans = holds;
    }
}
