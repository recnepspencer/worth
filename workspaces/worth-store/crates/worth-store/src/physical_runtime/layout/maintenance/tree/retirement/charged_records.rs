use std::{collections::BTreeSet, mem::size_of, num::NonZeroU64};

use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{MaintenancePhysicalAllocation, ServingPhysicalRuntime};

use super::super::replacements::{retained_capacity_charge, RETAINED_BACKING_BYTES};
use super::{record_budget::RetirementRecordBudget, PhysicalLayoutMaintenanceFailure};

// At most 131072 keys give eight BTree allocation levels at minimum branching
// six. A record key is <=32 bytes: eleven keys plus twelve edges and node
// bookkeeping fit 512 bytes. Eight split/path nodes plus the four directory
// family sets (one node each) and backing controls fit this fixed allowance.
const SET_HEADROOM_BYTES: u64 = 8192;
// Settled nodes amortize below 512/5 bytes per key; 160 preserves the admitted
// geometry with headroom for partially populated nodes outside the split path.
const SET_RECORD_BYTES: u64 = 160;
const _: () = assert!(size_of::<PersistedRecordIdentity>() <= 32);

pub(super) struct ChargedRetirementRecords<'runtime> {
    records: BTreeSet<PersistedRecordIdentity>,
    budget: RetirementRecordBudget,
    reserved_records: usize,
    allocation: MaintenancePhysicalAllocation<'runtime>,
}

impl<'runtime> ChargedRetirementRecords<'runtime> {
    pub(super) fn admit(
        runtime: &'runtime ServingPhysicalRuntime,
        budget: RetirementRecordBudget,
    ) -> Result<Self, PhysicalLayoutMaintenanceFailure> {
        let allocation = runtime
            .physical_allocations()
            .admit_maintenance(
                NonZeroU64::new(SET_HEADROOM_BYTES + RETAINED_BACKING_BYTES).unwrap(),
            )
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
        Ok(Self::from_allocation(budget, allocation))
    }

    fn from_allocation(
        budget: RetirementRecordBudget,
        allocation: MaintenancePhysicalAllocation<'runtime>,
    ) -> Self {
        Self {
            records: BTreeSet::new(),
            budget,
            reserved_records: 0,
            allocation,
        }
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.records.len()
    }
    pub(super) fn contains(&self, record: &PersistedRecordIdentity) -> bool {
        self.records.contains(record)
    }

    pub(super) fn insert(
        &mut self,
        record: PersistedRecordIdentity,
    ) -> Result<(), PhysicalLayoutMaintenanceFailure> {
        let present = self.records.contains(&record);
        self.budget
            .before_unique_insert(self.records.len(), present)?;
        if present {
            return Ok(());
        }
        let required = self.records.len() + 1;
        if required > self.reserved_records {
            let capacity = required
                .max(self.reserved_records.saturating_mul(2))
                .min(self.budget.maximum());
            let bytes = collection_charge(capacity)?;
            self.allocation
                .try_resize(bytes)
                .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
            self.reserved_records = capacity;
        }
        self.records.insert(record);
        Ok(())
    }

    pub(super) fn extend(
        &mut self,
        records: &[PersistedRecordIdentity],
    ) -> Result<(), PhysicalLayoutMaintenanceFailure> {
        for record in records {
            self.insert(*record)?;
        }
        Ok(())
    }

    pub(super) fn into_retained_parts(
        mut self,
    ) -> Result<
        (
            Vec<PersistedRecordIdentity>,
            MaintenancePhysicalAllocation<'runtime>,
        ),
        PhysicalLayoutMaintenanceFailure,
    > {
        self.budget.before_copy(self.records.len())?;
        let retained = retained_capacity_charge(self.records.len())?;
        let peak = self
            .allocation
            .bytes()
            .checked_add(retained)
            .ok_or(PhysicalLayoutMaintenanceFailure::RetirementLimit)?;
        self.allocation
            .try_resize(peak)
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
        let mut records = Vec::new();
        records
            .try_reserve_exact(self.records.len())
            .map_err(|_| PhysicalLayoutMaintenanceFailure::RetirementLimit)?;
        records.extend(self.records);
        // Consuming the iterator disposed every set node. Only the actual Vec
        // backing and its later shared prepared-directory owner remain live.
        self.allocation
            .try_resize(retained_capacity_charge(records.capacity())?)
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
        Ok((records, self.allocation))
    }
}

fn collection_charge(capacity: usize) -> Result<u64, PhysicalLayoutMaintenanceFailure> {
    (capacity as u64)
        .checked_mul(SET_RECORD_BYTES)
        .and_then(|bytes| bytes.checked_add(SET_HEADROOM_BYTES + RETAINED_BACKING_BYTES))
        .ok_or(PhysicalLayoutMaintenanceFailure::RetirementLimit)
}

#[cfg(test)]
mod tests;
