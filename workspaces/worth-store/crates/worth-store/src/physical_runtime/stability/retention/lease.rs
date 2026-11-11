use std::sync::Arc;

use super::super::{PhysicalProtectedRootObservation, PhysicalReadProtectionDenial};
use super::RootProtectionRegistry;

#[derive(Debug)]
pub(in crate::physical_runtime) enum PhysicalRootReadRetirementDenial {
    Busy,
    Execution(worth_execution::WorkCeilingDenial),
}

/// Owns a reference to one registered reader acquisition, not to the runtime.
pub(in crate::physical_runtime) struct PhysicalRootReadLease {
    registry: Arc<RootProtectionRegistry>,
    slot: usize,
    binding: PhysicalProtectedRootObservation,
    armed: bool,
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
            armed: true,
        }
    }

    pub(in crate::physical_runtime) fn require_live(
        &self,
    ) -> Result<(), PhysicalReadProtectionDenial> {
        self.registry.require_live(self.binding)
    }

    pub(in crate::physical_runtime) fn try_require_live(
        &self,
    ) -> Result<(), crate::physical_runtime::PhysicalCheckpointReadProtectionDenial> {
        self.registry.try_require_live(self.binding)
    }

    pub(in crate::physical_runtime) const fn observation(
        &self,
    ) -> PhysicalProtectedRootObservation {
        self.binding
    }

    pub(in crate::physical_runtime) fn try_retire(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), PhysicalRootReadRetirementDenial> {
        if self.armed {
            self.registry
                .try_release(self.slot, self.binding.root(), &mut self.armed, request)?;
        }
        Ok(())
    }
}

impl Clone for PhysicalRootReadLease {
    fn clone(&self) -> Self {
        assert!(self.armed, "retired read protection cannot be shared");
        self.registry.share(self.slot, self.binding.root());
        Self {
            registry: Arc::clone(&self.registry),
            slot: self.slot,
            binding: self.binding,
            armed: true,
        }
    }
}

impl Drop for PhysicalRootReadLease {
    fn drop(&mut self) {
        if self.armed {
            self.registry.release(self.slot, self.binding.root());
        }
    }
}
