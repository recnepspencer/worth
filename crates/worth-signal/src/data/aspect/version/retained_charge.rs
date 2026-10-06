use super::{AspectVersion, PartitionVersionOverrides, PathVersions};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for AspectVersion {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { slots: _ } = self;
        Ok(Charge::ZERO)
    }
}

impl RetainedStorageMeasurement for PartitionVersionOverrides {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { baseline: _, paths } = self;
        paths.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for PathVersions {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        Ok(Charge::ZERO)
    }
}

impl PartitionVersionOverrides {
    /// Worst new prefix-tree allocation admitted by a checked result heap.
    /// Every depth-d region owns its Vec slot, at least d String slots and
    /// d nonempty segment bytes. The ratio of prefixes to those bytes grows
    /// with depth, so the deepest valid path gives the maximum prefix count.
    pub(crate) fn evaluation_growth_ceiling(result_heap_bytes: u64) -> Result<Charge, Denial> {
        use crate::data::output::{ChangedRegion, ScopePath};
        use crate::data::retained_storage::btree_structure_charge;
        let depth = ScopePath::MAX_DEPTH as u64;
        let slot = std::mem::size_of::<ChangedRegion>() as u64;
        let segment = (std::mem::size_of::<String>() as u64)
            .checked_add(1)
            .ok_or(Denial::ChargeOverflow)?;
        let minimum_deep_region = slot
            .checked_add(depth.checked_mul(segment).ok_or(Denial::ChargeOverflow)?)
            .ok_or(Denial::ChargeOverflow)?;
        let paths = result_heap_bytes
            .checked_mul(depth)
            .ok_or(Denial::ChargeOverflow)?
            / minimum_deep_region;
        if paths == 0 {
            return Ok(Charge::ZERO);
        }
        let paths = usize::try_from(paths).map_err(|_| Denial::ChargeOverflow)?;
        let prefix_slots = (paths as u64)
            .checked_mul(depth.checked_add(1).ok_or(Denial::ChargeOverflow)?)
            .and_then(|count| count.checked_div(2))
            .and_then(|count| count.checked_mul(std::mem::size_of::<String>() as u64))
            .ok_or(Denial::ChargeOverflow)?;
        let copied_bytes = result_heap_bytes
            .checked_mul(depth)
            .ok_or(Denial::ChargeOverflow)?;
        btree_structure_charge::<super::ScopePath, PathVersions>(paths)?
            .checked_add(Charge::from_bytes(prefix_slots))?
            .checked_add(Charge::from_bytes(copied_bytes))
    }

    /// Bound new tree nodes and copied scope keys before applying output regions.
    /// Existing payload clones are admitted by the retained node draft owner.
    pub(crate) fn evaluation_growth_charge(
        &self,
        regions: &[crate::data::output::ChangedRegion],
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        use crate::data::retained_storage::btree_structure_charge;
        work.reserve_visits(regions.len())?;
        let count = regions.iter().try_fold(0usize, |count, region| {
            count
                .checked_add(region.path().depth())
                .ok_or(Denial::ChargeOverflow)
        })?;
        let mut charge = if count == 0 {
            Charge::ZERO
        } else {
            btree_structure_charge::<super::ScopePath, PathVersions>(count)?
        };
        for region in regions {
            // Prefix k clones exactly k String slots. Segment i occurs in
            // every prefix from i+1 through depth; charge its source capacity
            // as a safe ceiling for each cloned String's requested storage.
            let depth = region.path().depth();
            let slots = depth
                .checked_mul(depth.checked_add(1).ok_or(Denial::ChargeOverflow)?)
                .and_then(|n| n.checked_div(2))
                .ok_or(Denial::ChargeOverflow)?;
            work.reserve_visits(depth.checked_add(slots).ok_or(Denial::ChargeOverflow)?)?;
            let mut copied_bytes = 0usize;
            for (index, segment) in region.path().segments().iter().enumerate() {
                copied_bytes = copied_bytes
                    .checked_add(
                        segment
                            .capacity()
                            .checked_mul(depth - index)
                            .ok_or(Denial::ChargeOverflow)?,
                    )
                    .ok_or(Denial::ChargeOverflow)?;
            }
            charge = charge
                .checked_add(Charge::capacity::<String>(slots)?)?
                .checked_add(Charge::capacity::<u8>(copied_bytes)?)?;
        }
        Ok(charge)
    }
}
