use super::*;

impl PhysicalCurrentRootOwner {
    /// Constructor-only caller has admitted the historical root and checked its
    /// exact extent route against the authenticated retained copy intent.
    pub(in crate::physical_runtime) fn protect_recovered_copy_source(
        &self,
        root: &DurablePhysicalRootManifest,
    ) -> Result<crate::physical_runtime::stability::PhysicalRootReadLease, ()> {
        let state = self.lock_publication_state();
        if root.generation() > state.current_root.generation() {
            return Err(());
        }
        self.read_protection.capture(root).map_err(|_| ())
    }
}
