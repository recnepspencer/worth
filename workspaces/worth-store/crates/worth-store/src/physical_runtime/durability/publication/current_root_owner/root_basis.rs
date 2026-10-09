use worth_store_physical_format::{DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest};

use super::PhysicalCurrentRootOwner;

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn snapshot(
        &self,
    ) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
        let state = self.lock_publication_state();
        (state.current_root.clone(), state.free_space.clone())
    }

    pub(in crate::physical_runtime) fn into_recovery_root_basis(
        self,
    ) -> crate::physical_runtime::PhysicalRecoveryRootBasis {
        let state = self.lock_publication_state();
        crate::physical_runtime::PhysicalRecoveryRootBasis::new(
            state.current_root.clone(),
            state.previous_root.clone(),
            state.namespace_evidence,
        )
    }
}
