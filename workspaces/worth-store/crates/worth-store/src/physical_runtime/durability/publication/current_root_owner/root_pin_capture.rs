//! O(1) root pin capture under the current-root lock and reclaim fence.
use worth_store_physical_format::DurablePhysicalRootManifest;

use super::PhysicalCurrentRootOwner;
use crate::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId};

impl PhysicalCurrentRootOwner {
    /// Pin the current root exactly as a reader capture would protect it.
    ///
    /// The publication-state lock is the lock retirement claims take, so no
    /// claim can observe the root unprotected between selection and pinning.
    /// Cost: O(live pins) kernel bookkeeping; no record scan or copy.
    pub(in crate::physical_runtime) fn pin_current_root(
        &self,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        payload: Vec<u8>,
    ) -> Result<(RootPinId, DurablePhysicalRootManifest), RootPinDenial> {
        let state = self.lock_publication_state();
        if self.lock_reclaim().is_some() {
            return Err(RootPinDenial::ReclaimFenced);
        }
        let root = state.current_root.clone();
        let pin = self
            .read_protection
            .pins()
            .pin(&root, parent, policy, payload)?;
        Ok((pin, root))
    }
}
