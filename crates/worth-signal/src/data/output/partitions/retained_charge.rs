use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{
    CanonicalChangedRegions, ChangedRegion, InternedScopePath, PartitionSubscription,
    PartitionToken,
};
use super::{PartitionInterner, PartitionTokenId};
use crate::data::output::ScopePath;

impl RetainedStorageMeasurement for PartitionTokenId {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self(_) = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for InternedScopePath {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for PartitionInterner {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            segments,
            segment_lookup,
        } = self;
        Charge::ZERO
            .checked_add(segments.retained_heap_charge(work)?)?
            .checked_add(segment_lookup.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for PartitionToken {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.0.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ScopePath {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.storage().retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for PartitionSubscription {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.path().retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for ChangedRegion {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.path().retained_heap_charge(work)
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
            segments,
            segment_lookup,
        } = self;
        RetainedStorageForkCharge::unchanged(Charge::ZERO)
            .checked_add(segments.prepare_fork_charge(work)?)?
            .checked_add(segment_lookup.prepare_fork_charge(work)?)
    }
}
