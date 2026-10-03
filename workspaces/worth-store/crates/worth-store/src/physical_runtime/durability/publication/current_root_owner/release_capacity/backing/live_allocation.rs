//! Bound admission and shared custody of the selected ledger's live backing.

use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalOperationAllocationScope};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::super::ReleaseCertificateCapacityDenial as Denial;
use crate::physical_runtime::lifecycle::{LifecycleState, ObservedLifecyclePhase};
use crate::physical_runtime::record_serving::RecordFramePorts;
use crate::physical_runtime::{
    LifecycleGeneration, PhysicalRecoveryAllocationAdmission, PhysicalRecoveryRejoinResidentDenial,
    PhysicalScopedAllocationFailure, RuntimeIdentity,
};

/// Constructed only from the publication foundation's existing pool and binding.
/// It cannot mint an unbound grant or borrow a temporary serving port.
pub(in crate::physical_runtime) struct ReleasePublicationAllocationOwner {
    ports: RecordFramePorts,
    store: StableStoreIdentity,
    maximum: u64,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    lifecycle: Arc<LifecycleState>,
}

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct LiveReleaseAllocation
{
    store: StableStoreIdentity,
    runtime: RuntimeIdentity,
    generation: LifecycleGeneration,
    grant: Mutex<OperationAllocationGrant>,
}

impl ReleasePublicationAllocationOwner {
    pub(in crate::physical_runtime) fn new(
        ports: RecordFramePorts,
        allocation: PhysicalRecoveryAllocationAdmission,
        runtime: RuntimeIdentity,
        generation: LifecycleGeneration,
        lifecycle: Arc<LifecycleState>,
    ) -> Self {
        Self {
            ports,
            store: allocation.store_identity(),
            maximum: allocation.byte_limit(),
            runtime,
            generation,
            lifecycle,
        }
    }

    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn fund(
        &self,
        custody: &mut Option<Arc<LiveReleaseAllocation>>,
        ceiling: PhysicalRecoveryAllocationAdmission,
        required: u64,
    ) -> Result<(), Denial> {
        let active = self.lifecycle.snapshot();
        if ceiling.store_identity() != self.store
            || self.ports.store_identity() != self.store
            || ceiling.byte_limit() > self.maximum
            || active.generation != self.generation
            || active.phase != ObservedLifecyclePhase::RecordServing
        {
            return Err(Denial::SelectedFactMismatch);
        }
        if required > ceiling.byte_limit() {
            return Err(Denial::Resident(
                PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                    required,
                    admitted: ceiling.byte_limit(),
                },
            ));
        }
        let bytes = NonZeroU64::new(required).ok_or(Denial::CapacityExhausted)?;
        if let Some(custody) = custody {
            if custody.store != self.store
                || custody.runtime != self.runtime
                || custody.generation != self.generation
            {
                return Err(Denial::SelectedFactMismatch);
            }
            let mut grant = custody
                .grant
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            // Cancellation, commit and checkpoint drains retain spare capacity.
            // Never give away backing that another funded owner can still hold.
            if required > grant.bytes() {
                grant
                    .try_resize(bytes.get())
                    .map_err(|cause| self.denial(cause))?;
            }
        } else {
            let grant = self
                .ports
                .begin_operation(PhysicalOperationAllocationScope::Recovery, bytes)
                .map_err(|cause| self.denial(cause))?;
            if grant.observation().store() != self.store {
                return Err(Denial::SelectedFactMismatch);
            }
            *custody = Some(Arc::new(LiveReleaseAllocation {
                store: self.store,
                runtime: self.runtime,
                generation: self.generation,
                grant: Mutex::new(grant),
            }));
        }
        Ok(())
    }

    fn denial(&self, cause: worth_store_buffer_pool::PhysicalResidencyDenial) -> Denial {
        Denial::Resident(PhysicalRecoveryRejoinResidentDenial::OperationAllocation(
            PhysicalScopedAllocationFailure::from_denial(cause, self.generation),
        ))
    }
}

impl LiveReleaseAllocation {
    #[cfg(test)]
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn bytes(
        &self,
    ) -> u64 {
        self.grant
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .bytes()
    }
}

/// Shared object and the Arc strong/weak control words are funded before Arc::new.
pub(in crate::physical_runtime::durability::publication::current_root_owner) const SHARED_CUSTODY_BYTES: u64 =
    (std::mem::size_of::<LiveReleaseAllocation>() + 2 * std::mem::size_of::<usize>()) as u64;

/// The pending fence can outlive the root owner's selected ledger. Keep its
/// actual scratch and charge together; byte backing drops before custody.
pub(in crate::physical_runtime::durability::publication::current_root_owner) struct FundedRootFrame
{
    pub(in crate::physical_runtime::durability::publication::current_root_owner) bytes: Vec<u8>,
    _custody: Arc<LiveReleaseAllocation>,
}

impl FundedRootFrame {
    pub(super) fn new(bytes: Vec<u8>, custody: Arc<LiveReleaseAllocation>) -> Self {
        Self {
            bytes,
            _custody: custody,
        }
    }
}
