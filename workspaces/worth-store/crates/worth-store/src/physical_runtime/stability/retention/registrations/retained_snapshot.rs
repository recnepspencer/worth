use std::sync::Arc;

use worth_store_physical_format::DurablePhysicalRootManifest;

use crate::physical_runtime::{
    lifecycle::ObservedLifecyclePhase, stability::PhysicalRootReadLease,
    PhysicalProtectedRootObservation, PhysicalReadProtectionDenial,
};

use super::RootProtectionRegistry;

impl RootProtectionRegistry {
    /// Shares every live registration while holding the registry lock. The
    /// returned leases keep old manifests protected throughout inspection.
    pub(in crate::physical_runtime) fn snapshot_retained_roots(
        self: &Arc<Self>,
        maximum_roots: usize,
    ) -> Result<
        Vec<(DurablePhysicalRootManifest, PhysicalRootReadLease)>,
        PhysicalReadProtectionDenial,
    > {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.revoked || self.lifecycle.snapshot().phase != ObservedLifecyclePhase::RecordServing
        {
            return Err(PhysicalReadProtectionDenial::Revoked);
        }
        if state.roots.len() > maximum_roots {
            return Err(PhysicalReadProtectionDenial::RetainedRootLimit);
        }
        let mut captured = Vec::new();
        captured
            .try_reserve_exact(state.roots.len())
            .map_err(|_| PhysicalReadProtectionDenial::MetadataUnavailable)?;
        if state
            .slots
            .iter()
            .any(|slot| slot.root.is_some() && slot.owners == u64::MAX)
        {
            return Err(PhysicalReadProtectionDenial::ProtectionLimit);
        }
        for slot in 0..state.slots.len() {
            let Some(cell) = state.slots[slot].root else {
                continue;
            };
            if captured.iter().any(
                |(manifest, _): &(DurablePhysicalRootManifest, PhysicalRootReadLease)| {
                    manifest.root_cell() == cell
                },
            ) {
                continue;
            }
            let manifest = state
                .roots
                .get_mut(&cell.generation().get())
                .expect("live slot has indexed root")
                .manifest
                .clone();
            state.slots[slot].owners += 1;
            let binding = PhysicalProtectedRootObservation::new(
                self.runtime,
                self.lifecycle.snapshot().generation,
                cell,
            );
            captured.push((
                manifest,
                PhysicalRootReadLease::new(Arc::clone(self), slot, binding),
            ));
        }
        Ok(captured)
    }
}
