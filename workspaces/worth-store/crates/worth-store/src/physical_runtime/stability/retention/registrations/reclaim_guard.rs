use super::{PhysicalProtectedRootObservation, RootProtectionRegistry};

impl RootProtectionRegistry {
    /// Reclaim inspection owns exactly one acquisition. Its descendants share
    /// that slot; every other acquisition at or below the source is external.
    /// The caller holds the current-root lock, so no new capture can race the
    /// check with installation of the reclaim fence.
    pub(in crate::physical_runtime) fn external_root_at_or_below(
        &self,
        inspector: PhysicalProtectedRootObservation,
        source_generation: u64,
    ) -> bool {
        if inspector.runtime() != self.runtime {
            return true;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.index_probes = state.index_probes.saturating_add(1);
        let mut inspector_present = false;
        let (visited, external) = state.roots.any_entry(|generation, protected| {
            if *generation > source_generation {
                return false;
            }
            if protected.manifest.root_cell() == inspector.root() {
                inspector_present = protected.acquisitions > 0;
                protected.acquisitions > 1
            } else {
                true
            }
        });
        state.examined_entries = state.examined_entries.saturating_add(visited);
        external || !inspector_present
    }
}
