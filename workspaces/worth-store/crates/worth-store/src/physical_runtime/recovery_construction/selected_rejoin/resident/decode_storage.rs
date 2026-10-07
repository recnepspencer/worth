//! Exact decoder-request admission in the original native Recovery pool.
//! The accumulated grant remains live until the decoded projection is used.

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_format::{PhysicalRecoveryDecodeFailure, PhysicalRecoveryDecodeStorage};

use super::super::SelectedMediaRejoinDenial as Denial;
use super::{PhysicalRecoveryRejoinResidentDenial as ResidentDenial, StoreRejoinResidentLedger};
use crate::physical_runtime::PhysicalRecoveryReadAllocation;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct RejoinDecodeStorage<
    'scope,
    'owner,
> {
    window: &'scope PhysicalRecoveryReadAllocation<'owner>,
    resident: &'scope mut StoreRejoinResidentLedger,
    grant: Option<OperationAllocationGrant>,
    charged: u64,
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct RejoinDecodeCharge {
    grant: Option<OperationAllocationGrant>,
    charged: u64,
}

/// Field order is the failure-path order: decoded owning values drop before
/// the native reservation that admitted them, including under `?`.
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct RejoinDecoded<T> {
    data: Option<T>,
    charge: Option<RejoinDecodeCharge>,
}

impl<'scope, 'owner> RejoinDecodeStorage<'scope, 'owner> {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn new(
        window: &'scope PhysicalRecoveryReadAllocation<'owner>,
        resident: &'scope mut StoreRejoinResidentLedger,
    ) -> Self {
        Self {
            window,
            resident,
            grant: None,
            charged: 0,
        }
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_charge(
        mut self,
    ) -> RejoinDecodeCharge {
        RejoinDecodeCharge {
            grant: self.grant.take(),
            charged: std::mem::replace(&mut self.charged, 0),
        }
    }
}

impl Drop for RejoinDecodeStorage<'_, '_> {
    fn drop(&mut self) {
        self.resident.release(self.charged);
    }
}

impl PhysicalRecoveryDecodeStorage for RejoinDecodeStorage<'_, '_> {
    type Denial = Denial;

    fn admit_allocation(&mut self, bytes: u64) -> Result<(), Self::Denial> {
        let next = self
            .charged
            .checked_add(bytes)
            .ok_or(Denial::BoundExceeded)?;
        self.resident.transient(bytes).map_err(Denial::Resident)?;
        match self.grant.as_mut() {
            Some(grant) => grant
                .try_resize(next)
                .map_err(|cause| Denial::Resident(self.window.map_allocation_denial(cause)))?,
            None => self.grant = self.window.reserve_owned(next).map_err(Denial::Resident)?,
        }
        self.resident.retain(bytes).map_err(Denial::Resident)?;
        self.charged = next;
        Ok(())
    }
}

impl RejoinDecodeCharge {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn release(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) {
        resident.release(self.charged);
        drop(self.grant);
    }
}

impl<T> RejoinDecoded<T> {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn new(
        data: T,
        charge: RejoinDecodeCharge,
    ) -> Self {
        Self {
            data: Some(data),
            charge: Some(charge),
        }
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn with_data<R, E>(
        mut self,
        resident: &mut StoreRejoinResidentLedger,
        consume: impl FnOnce(&T, &mut StoreRejoinResidentLedger) -> Result<R, E>,
    ) -> Result<R, E> {
        let result = consume(
            self.data.as_ref().expect("decoded owner retains data"),
            resident,
        );
        drop(self.data.take());
        self.charge
            .take()
            .expect("decoded owner retains admission")
            .release(resident);
        result
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn decode_denial(
    failure: PhysicalRecoveryDecodeFailure<Denial>,
) -> Denial {
    match failure {
        PhysicalRecoveryDecodeFailure::Allocation(cause) => cause,
        PhysicalRecoveryDecodeFailure::AllocationFailed { requested, cause } => {
            Denial::Resident(ResidentDenial::Allocation { requested, cause })
        }
        PhysicalRecoveryDecodeFailure::AllocatorExceededReservation { requested, actual } => {
            Denial::Resident(ResidentDenial::AllocatorExceededReservation { requested, actual })
        }
        PhysicalRecoveryDecodeFailure::SizeOverflow => Denial::BoundExceeded,
        PhysicalRecoveryDecodeFailure::Canonical(cause) => Denial::CanonicalRedo(cause),
        PhysicalRecoveryDecodeFailure::Projection(cause) => Denial::RecoveryProjection(cause),
    }
}

#[cfg(test)]
#[path = "decode_storage/tests.rs"]
mod tests;
