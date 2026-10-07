use std::{mem::size_of, num::NonZeroU64};

use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{MaintenancePhysicalAllocation, ServingPhysicalRuntime};

use super::{PhysicalLayoutMaintenanceFailure, MAXIMUM_HEIGHT, MAXIMUM_RETIREMENT_RECORDS};

// Funds the Vec header, scoped/retained grant, shared-directory Arc control
// words and alignment. The backing is retained through preparation clones.
pub(super) const RETAINED_BACKING_BYTES: u64 = 256;

pub(super) fn retained_capacity_charge(
    capacity: usize,
) -> Result<u64, PhysicalLayoutMaintenanceFailure> {
    (capacity as u64)
        .checked_mul(size_of::<PersistedRecordIdentity>() as u64)
        .and_then(|bytes| bytes.checked_add(RETAINED_BACKING_BYTES))
        .ok_or(PhysicalLayoutMaintenanceFailure::RetirementLimit)
}

pub(super) struct ChargedReplacementRecords<'runtime> {
    records: Vec<PersistedRecordIdentity>,
    allocation: MaintenancePhysicalAllocation<'runtime>,
}

impl<'runtime> ChargedReplacementRecords<'runtime> {
    pub(super) fn admit(
        runtime: &'runtime ServingPhysicalRuntime,
    ) -> Result<Self, PhysicalLayoutMaintenanceFailure> {
        let allocation = runtime
            .physical_allocations()
            .admit_maintenance(NonZeroU64::new(RETAINED_BACKING_BYTES).unwrap())
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
        Ok(Self::from_allocation(allocation))
    }

    fn from_allocation(allocation: MaintenancePhysicalAllocation<'runtime>) -> Self {
        Self {
            records: Vec::new(),
            allocation,
        }
    }

    pub(super) fn belongs_to(&self, runtime: &ServingPhysicalRuntime) -> bool {
        self.allocation.store_identity() == runtime.store_identity()
            && self.allocation.runtime_identity() == runtime.runtime_identity()
            && self.allocation.store_generation()
                == runtime.residency_observation().store_generation()
    }

    pub(super) fn prepare_insertion(&mut self) -> Result<(), PhysicalLayoutMaintenanceFailure> {
        let required = self
            .records
            .len()
            .checked_add(usize::from(MAXIMUM_HEIGHT))
            .filter(|count| *count <= MAXIMUM_RETIREMENT_RECORDS)
            .ok_or(PhysicalLayoutMaintenanceFailure::RetirementLimit)?;
        if required <= self.records.capacity() {
            return Ok(());
        }
        let prospective = required
            .max(self.records.capacity().saturating_mul(2))
            .min(MAXIMUM_RETIREMENT_RECORDS);
        let retained = retained_capacity_charge(self.records.capacity())?;
        // A reallocating allocator may keep the old backing until the new
        // allocation succeeds. Acquire that peak before asking it to grow.
        let peak = retained
            .checked_add(retained_capacity_charge(prospective)?)
            .ok_or(PhysicalLayoutMaintenanceFailure::RetirementLimit)?;
        self.allocation
            .try_resize(peak)
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
        if self
            .records
            .try_reserve_exact(prospective - self.records.len())
            .is_err()
        {
            self.allocation
                .try_resize(retained)
                .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
            return Err(PhysicalLayoutMaintenanceFailure::RetirementLimit);
        }
        self.allocation
            .try_resize(retained_capacity_charge(self.records.capacity())?)
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)
    }

    pub(super) fn append_prepared(&mut self, path: ReplacementPath) {
        assert!(self.records.capacity() - self.records.len() >= usize::from(path.len));
        self.records.extend(
            path.records
                .into_iter()
                .take(usize::from(path.len))
                .map(|record| record.expect("initialized replacement path prefix")),
        );
    }

    pub(super) fn records(&self) -> &[PersistedRecordIdentity] {
        &self.records
    }
}

pub(super) struct ReplacementPath {
    records: [Option<PersistedRecordIdentity>; MAXIMUM_HEIGHT as usize],
    len: u8,
}

impl ReplacementPath {
    pub(super) const fn empty() -> Self {
        Self {
            records: [None; MAXIMUM_HEIGHT as usize],
            len: 0,
        }
    }

    pub(super) fn one(record: PersistedRecordIdentity) -> Self {
        let mut path = Self::empty();
        path.push(record);
        path
    }

    pub(super) fn push(&mut self, record: PersistedRecordIdentity) {
        self.records[usize::from(self.len)] = Some(record);
        self.len += 1;
    }
}

#[cfg(test)]
mod tests;
