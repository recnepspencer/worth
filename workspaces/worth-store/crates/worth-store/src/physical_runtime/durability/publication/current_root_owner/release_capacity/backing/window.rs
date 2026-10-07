//! Allocation-first bridge from the original numerical ceiling to live pool custody.

use super::super::ReleaseCertificateCapacityDenial as Denial;
use super::{LiveReleaseAllocation, ReleasePublicationAllocationOwner};
use crate::physical_runtime::{
    recovery_residency::StoreRejoinResidentLedger, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryRejoinResidentDenial,
};
use std::sync::Arc;

pub(in crate::physical_runtime::durability::publication::current_root_owner::release_capacity)
struct LiveBackingWindow
<'a> {
    owner: &'a ReleasePublicationAllocationOwner,
    custody: &'a mut Option<Arc<LiveReleaseAllocation>>,
    ceiling: PhysicalRecoveryAllocationAdmission,
    resident: StoreRejoinResidentLedger,
}

impl<'a> LiveBackingWindow<'a> {
    pub(in crate::physical_runtime::durability::publication::current_root_owner::release_capacity) fn new(
        owner: &'a ReleasePublicationAllocationOwner,
        custody: &'a mut Option<Arc<LiveReleaseAllocation>>,
        ceiling: PhysicalRecoveryAllocationAdmission,
        already_live: u64,
    ) -> Result<Self, Denial> {
        let resident = StoreRejoinResidentLedger::from_retained_with_limit(
            ceiling,
            already_live,
            ceiling.byte_limit(),
        )
        .map_err(Denial::Resident)?;
        owner.fund(custody, ceiling, already_live)?;
        Ok(Self {
            owner,
            custody,
            ceiling,
            resident,
        })
    }

    pub(in crate::physical_runtime::durability::publication::current_root_owner::release_capacity) fn grow_vec<
        T,
    >(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Denial> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| self.overflow())?;
        if needed > values.capacity() {
            self.fund_additional::<T>(needed)?;
        }
        self.resident
            .grow_vec(values, additional)
            .map_err(Denial::Resident)?;
        self.owner
            .fund(self.custody, self.ceiling, self.resident.peak())
    }

    pub(in crate::physical_runtime::durability::publication::current_root_owner::release_capacity) fn reserve_vec<
        T,
    >(
        &mut self,
        count: usize,
    ) -> Result<Vec<T>, Denial> {
        self.fund_additional::<T>(count)?;
        let values = self.resident.reserve_vec(count).map_err(Denial::Resident)?;
        self.owner
            .fund(self.custody, self.ceiling, self.resident.peak())?;
        Ok(values)
    }

    fn fund_additional<T>(&mut self, count: usize) -> Result<(), Denial> {
        let bytes = count
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| self.overflow())?;
        self.resident.transient(bytes).map_err(Denial::Resident)?;
        let required = self
            .resident
            .used()
            .checked_add(bytes)
            .ok_or_else(|| self.overflow())?;
        self.owner.fund(self.custody, self.ceiling, required)
    }

    fn overflow(&self) -> Denial {
        Denial::Resident(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
            admitted: self.ceiling.byte_limit(),
        })
    }
}
