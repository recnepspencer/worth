use std::sync::Arc;

use super::super::{PhysicalProtectedRootObservation, PhysicalReadProtectionDenial};
use super::RootProtectionRegistry;

/// Owns a reference to one registered reader acquisition, not to the runtime.
pub(in crate::physical_runtime) struct PhysicalRootReadLease {
    registry: Arc<RootProtectionRegistry>,
    slot: usize,
    binding: PhysicalProtectedRootObservation,
}

impl PhysicalRootReadLease {
    pub(super) fn new(
        registry: Arc<RootProtectionRegistry>,
        slot: usize,
        binding: PhysicalProtectedRootObservation,
    ) -> Self {
        Self {
            registry,
            slot,
            binding,
        }
    }

    pub(in crate::physical_runtime) fn require_live(
        &self,
    ) -> Result<(), PhysicalReadProtectionDenial> {
        self.registry.require_live(self.binding)
    }

    pub(in crate::physical_runtime) const fn observation(
        &self,
    ) -> PhysicalProtectedRootObservation {
        self.binding
    }
}

impl Clone for PhysicalRootReadLease {
    fn clone(&self) -> Self {
        self.registry.share(self.slot, self.binding.root());
        Self {
            registry: Arc::clone(&self.registry),
            slot: self.slot,
            binding: self.binding,
        }
    }
}

impl Drop for PhysicalRootReadLease {
    fn drop(&mut self) {
        self.registry.release(self.slot, self.binding.root());
    }
}
