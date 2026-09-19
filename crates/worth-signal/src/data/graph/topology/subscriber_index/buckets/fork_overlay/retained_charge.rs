use super::{BucketDelta, ReverseSubscriptionFlat, ReverseSubscriptionStorage, SetDelta};
use crate::data::persistent_hash_map::PersistentHashMap;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageForkCharge, RetainedStorageForkPreparation,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageForkPreparation for ReverseSubscriptionStorage {
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        let source = self.retained_heap_charge(work)?;
        match self {
            Self::Exclusive(_) => {
                let retained = source
                    .checked_add(arc_allocation_charge::<
                        crate::data::retained_storage::RetainedStorageBacking<
                            ReverseSubscriptionFlat,
                        >,
                    >()?)?
                    .checked_add(PersistentHashMap::<
                        super::ProducerAspectKey,
                        BucketDelta,
                    >::empty_persistent_overlay_charge()?)?
                    .checked_add(PersistentHashMap::<
                        super::NodeId,
                        super::ForkConsumerMembershipChange,
                    >::empty_persistent_overlay_charge()?)?;
                RetainedStorageForkCharge::from_charges(source, retained)
            }
            Self::ForkShared { .. } => Ok(RetainedStorageForkCharge::unchanged(source)),
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
        Ok(Charge::ZERO
            .checked_add(buckets.retained_heap_charge(work)?)?
            .checked_add(by_consumer.retained_heap_charge(work)?)?)
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
        Ok(Charge::ZERO
            .checked_add(added.retained_heap_charge(work)?)?
            .checked_add(removed.retained_heap_charge(work)?)?
            .checked_add(retired_base_intervals.retained_heap_charge(work)?)?)
    }
}
impl RetainedStorageMeasurement for BucketDelta {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            all,
            unscoped,
            whole_partitions,
            exact_details,
            partition_scoped,
        } = self;
        Ok(Charge::ZERO
            .checked_add(all.retained_heap_charge(work)?)?
            .checked_add(unscoped.retained_heap_charge(work)?)?
            .checked_add(whole_partitions.retained_heap_charge(work)?)?
            .checked_add(exact_details.retained_heap_charge(work)?)?
            .checked_add(partition_scoped.retained_heap_charge(work)?)?)
    }
}
impl RetainedStorageMeasurement for ReverseSubscriptionStorage {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Exclusive(flat) => flat.retained_heap_charge(work),
            Self::ForkShared {
                base,
                bucket_changes,
                consumer_changes,
            } => Ok(base
                .retained_heap_charge(work)?
                .checked_add(bucket_changes.retained_heap_charge(work)?)?
                .checked_add(consumer_changes.retained_heap_charge(work)?)?),
        }
    }
}
