use super::fork_overlay::{BucketDelta, ReverseSubscriptionStorage};
use super::{
    ForkConsumerMembershipChange, NodeId, ProducerAspectKey, ReverseSubscriptionFlat,
    ReverseSubscriptionIndex,
};
use crate::data::persistent_hash_map::PersistentHashMap;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageBacking, RetainedStorageCharge as Charge,
    RetainedStorageForkGrowth, RetainedStorageForkGrowthDenial as Denial,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};

impl RetainedStorageForkGrowth for ReverseSubscriptionIndex {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self { storage, valid: _ } = self;
        let growth = match storage {
            ReverseSubscriptionStorage::Exclusive(_) => {
                arc_allocation_charge::<RetainedStorageBacking<ReverseSubscriptionFlat>>()?
                    .checked_add(PersistentHashMap::<
                        ProducerAspectKey,
                        BucketDelta,
                    >::empty_persistent_overlay_charge()?)?
                    .checked_add(PersistentHashMap::<
                        NodeId,
                        ForkConsumerMembershipChange,
                    >::empty_persistent_overlay_charge()?)?
            }
            ReverseSubscriptionStorage::ForkShared { .. } => Charge::ZERO,
        };
        work.reserve_visits(
            usize::try_from(growth.bytes())
                .map_err(|_| RetainedStoragePreparationDenial::ChargeOverflow)?,
        )?;
        Ok(growth)
    }
}
