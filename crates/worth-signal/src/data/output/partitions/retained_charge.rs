use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{CanonicalChangedRegions, ChangedRegion, PartitionSubscription, PartitionToken};
use super::{DetailTokenId, PartitionInterner, PartitionTokenId};

impl RetainedStorageMeasurement for PartitionTokenId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for DetailTokenId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for PartitionInterner {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            partitions,
            details,
            partition_lookup,
            detail_lookup,
        } = self;
        Ok(Charge::ZERO
            .checked_add(partitions.retained_heap_charge(work)?)?
            .checked_add(details.retained_heap_charge(work)?)?
            .checked_add(partition_lookup.retained_heap_charge(work)?)?
            .checked_add(detail_lookup.retained_heap_charge(work)?)?)
    }
}

impl RetainedStorageMeasurement for PartitionToken {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for PartitionSubscription {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            partition,
            detail,
            match_mode: _,
        } = self;
        Ok(Charge::ZERO
            .checked_add(partition.retained_heap_charge(work)?)?
            .checked_add(detail.retained_heap_charge(work)?)?)
    }
}

impl RetainedStorageMeasurement for ChangedRegion {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { partition, detail } = self;
        partition
            .retained_heap_charge(work)?
            .checked_add(detail.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for CanonicalChangedRegions {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        let Self { regions } = self;
        regions.retained_heap_charge(work)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for PartitionInterner {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let Self {
            partitions,
            details,
            partition_lookup,
            detail_lookup,
        } = self;
        Ok(RetainedStorageForkCharge::unchanged(Charge::ZERO)
            .checked_add(partitions.prepare_fork_charge(work)?)?
            .checked_add(details.prepare_fork_charge(work)?)?
            .checked_add(partition_lookup.prepare_fork_charge(work)?)?
            .checked_add(detail_lookup.prepare_fork_charge(work)?)?)
    }
}
