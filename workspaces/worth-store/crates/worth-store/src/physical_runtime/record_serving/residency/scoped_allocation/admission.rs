use std::num::NonZeroU64;

use crate::physical_runtime::{
    record_serving::residency::frame_ports::RecordFramePorts, LifecycleGeneration, RuntimeIdentity,
};

use super::{
    grant::StoreScopedAllocation,
    scope::{
        BlobScope, LayoutReadScope, MaintenanceScope, RecoveryScope, ScrubScope,
        StoreAllocationScope, VerificationScope,
    },
    BlobPhysicalAllocation, LayoutPhysicalAllocation, MaintenancePhysicalAllocation,
    PhysicalScopedAllocationFailure, RecoveryPhysicalAllocation, ScrubPhysicalAllocation,
    VerificationPhysicalAllocation,
};

/// Store-owned admission for successor physical-operation memory.
///
/// Each method charges one exact scope inside the admitted operation and total
/// byte envelopes. The resulting allocation borrows the serving runtime and
/// grants temporary-byte ownership only; successor policy and effects remain
/// with their owning feature.
pub struct PhysicalScopedAllocationAdmission<'runtime> {
    frame_ports: &'runtime RecordFramePorts,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
}

impl<'runtime> PhysicalScopedAllocationAdmission<'runtime> {
    pub(in crate::physical_runtime) const fn new(
        frame_ports: &'runtime RecordFramePorts,
        runtime: RuntimeIdentity,
        generation: LifecycleGeneration,
    ) -> Self {
        Self {
            frame_ports,
            runtime,
            generation,
        }
    }

    /// Charges temporary bytes to the Recovery scope.
    pub fn admit_recovery(
        &self,
        bytes: NonZeroU64,
    ) -> Result<RecoveryPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        self.admit::<RecoveryScope>(bytes)
            .map(RecoveryPhysicalAllocation::bind)
    }

    /// Charges temporary bytes to the Scrub scope.
    pub fn admit_scrub(
        &self,
        bytes: NonZeroU64,
    ) -> Result<ScrubPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        self.admit::<ScrubScope>(bytes)
            .map(ScrubPhysicalAllocation::bind)
    }

    /// Charges temporary bytes to the Maintenance scope.
    pub fn admit_maintenance(
        &self,
        bytes: NonZeroU64,
    ) -> Result<MaintenancePhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        self.admit::<MaintenanceScope>(bytes)
            .map(MaintenancePhysicalAllocation::bind)
    }

    /// Charges temporary bytes to the Verification scope.
    pub fn admit_verification(
        &self,
        bytes: NonZeroU64,
    ) -> Result<VerificationPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        self.admit::<VerificationScope>(bytes)
            .map(VerificationPhysicalAllocation::bind)
    }

    /// Charges temporary bytes to the Blob scope.
    pub fn admit_blob(
        &self,
        bytes: NonZeroU64,
    ) -> Result<BlobPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        self.admit::<BlobScope>(bytes)
            .map(BlobPhysicalAllocation::bind)
    }

    /// Charges decoded index-node scratch to the foreground-read envelope.
    pub fn admit_layout_read(
        &self,
        bytes: NonZeroU64,
    ) -> Result<LayoutPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        self.admit::<LayoutReadScope>(bytes)
            .map(LayoutPhysicalAllocation::bind)
    }

    fn admit<Scope: StoreAllocationScope>(
        &self,
        bytes: NonZeroU64,
    ) -> Result<StoreScopedAllocation<'runtime, Scope>, PhysicalScopedAllocationFailure> {
        let grant = self
            .frame_ports
            .begin_operation(Scope::VALUE, bytes)
            .map_err(|denial| {
                PhysicalScopedAllocationFailure::from_denial(denial, self.generation)
            })?;
        Ok(StoreScopedAllocation::from_pool_grant(
            grant,
            self.frame_ports,
            self.runtime,
            self.generation,
        ))
    }
}
