use super::{
    BucketDelta, ForkSubscriptions, ReverseSubscriptionFlat, ReverseSubscriptionStorage, SetDelta,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl RetainedStorageForkPreparation for ReverseSubscriptionStorage {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        let source = self.retained_heap_charge(work)?;
        match self {
            Self::Exclusive(_) => {
                let retained = source.checked_add(ForkSubscriptions::conversion_charge()?)?;
                RetainedStorageForkCharge::from_charges(source, retained)
            }
            Self::ForkShared(_) => Ok(RetainedStorageForkCharge::unchanged(source)),
        }
    }
}
impl RetainedStorageMeasurement for ReverseSubscriptionFlat {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            buckets,
            by_consumer,
        } = self;
        Charge::ZERO
            .checked_add(buckets.retained_heap_charge(work)?)?
            .checked_add(by_consumer.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for SetDelta {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            added,
            removed,
            retired_base_intervals,
        } = self;
        Charge::ZERO
            .checked_add(added.retained_heap_charge(work)?)?
            .checked_add(removed.retained_heap_charge(work)?)?
            .checked_add(retired_base_intervals.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for BucketDelta {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            all,
            unscoped,
            same_path,
            subtree_covering,
            subtree_members,
        } = self;
        Charge::ZERO
            .checked_add(all.retained_heap_charge(work)?)?
            .checked_add(unscoped.retained_heap_charge(work)?)?
            .checked_add(same_path.retained_heap_charge(work)?)?
            .checked_add(subtree_covering.retained_heap_charge(work)?)?
            .checked_add(subtree_members.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for ReverseSubscriptionStorage {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Exclusive(flat) => flat.retained_heap_charge(work),
            Self::ForkShared(overlay) => overlay.retained_heap_charge(work),
        }
    }
}
