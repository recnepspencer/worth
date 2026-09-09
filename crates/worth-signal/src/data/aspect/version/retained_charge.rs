use super::{AspectVersion, PartitionVersionOverrides};
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
        let Self {
            partitions,
            details,
        } = self;
        partitions
            .retained_heap_charge(work)?
            .checked_add(details.retained_heap_charge(work)?)
    }
}

impl PartitionVersionOverrides {
    /// Bound new tree nodes and copied scope keys before applying output regions.
    /// Existing payload clones are admitted by the retained node draft owner.
    pub(crate) fn evaluation_growth_charge(
        &self,
        regions: &[crate::data::output::ChangedRegion],
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        use crate::data::output::{PartitionSubscription, PartitionToken};
        use crate::data::retained_storage::btree_structure_charge;
        work.reserve_visits(regions.len())?;
        let partitions = self
            .partitions
            .len()
            .checked_add(regions.len())
            .ok_or(Denial::ChargeOverflow)?;
        let details = self
            .details
            .len()
            .checked_add(regions.len())
            .ok_or(Denial::ChargeOverflow)?;
        let mut charge =
            btree_structure_charge::<PartitionToken, AspectVersion>(partitions)?.checked_add(
                btree_structure_charge::<PartitionSubscription, AspectVersion>(details)?,
            )?;
        for region in regions {
            // Partition insertion, detail insertion, and the temporary range key.
            charge = charge
                .checked_add(
                    region
                        .partition
                        .retained_heap_charge(work)?
                        .checked_mul(3)?,
                )?
                .checked_add(region.detail.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}
