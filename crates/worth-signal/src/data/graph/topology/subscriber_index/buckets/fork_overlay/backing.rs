//! The persistent branch overlay shares one backing until its next edit.
use super::{
    BucketDelta, ForkConsumerMembershipChange, ProducerAspectKey, ReverseSubscriptionFlat,
};
use crate::data::handle::NodeId;
use crate::data::persistent_hash_map::PersistentHashMap;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageBacking, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial as Denial, SignalConditionalRetentionReservation as Custody,
};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in super::super) struct ForkSubscriptionMaps {
    pub(in super::super) base: Arc<RetainedStorageBacking<ReverseSubscriptionFlat>>,
    pub(in super::super) bucket_changes: PersistentHashMap<ProducerAspectKey, BucketDelta>,
    pub(in super::super) consumer_changes: PersistentHashMap<NodeId, ForkConsumerMembershipChange>,
}

struct Owner {
    maps: ForkSubscriptionMaps,
    _custody: Option<Custody>,
}

trait Backing: Send + Sync {
    fn maps(&self) -> &ForkSubscriptionMaps;
    fn maps_mut(&mut self) -> &mut ForkSubscriptionMaps;
}

impl Backing for Owner {
    fn maps(&self) -> &ForkSubscriptionMaps {
        &self.maps
    }

    fn maps_mut(&mut self) -> &mut ForkSubscriptionMaps {
        &mut self.maps
    }
}

#[derive(Clone)]
pub(in super::super) struct ForkSubscriptions(Arc<dyn Backing>);

impl ForkSubscriptions {
    pub(in super::super) fn conversion_charge() -> Result<Charge, Denial> {
        Self::allocation_charge()?
            .checked_add(arc_allocation_charge::<RetainedStorageBacking<ReverseSubscriptionFlat>>()?)?
            .checked_add(PersistentHashMap::<ProducerAspectKey, BucketDelta>::empty_persistent_overlay_charge()?)?
            .checked_add(PersistentHashMap::<NodeId, ForkConsumerMembershipChange>::empty_persistent_overlay_charge()?)
    }

    fn allocation_charge() -> Result<Charge, Denial> {
        arc_allocation_charge::<Owner>()
    }

    pub(in super::super) fn detached_payload_charge() -> Result<Charge, Denial> {
        // The ledger adds the reservation's embedded handle itself.
        Self::allocation_charge()?.checked_sub(Charge::capacity::<Custody>(1)?)
    }

    pub(in super::super) fn convert(
        flat: ReverseSubscriptionFlat,
        custody: Option<Custody>,
    ) -> Self {
        Self(Arc::new(Owner {
            maps: ForkSubscriptionMaps {
                base: Arc::new(RetainedStorageBacking::new(flat, custody)),
                bucket_changes: PersistentHashMap::new_persistent_overlay(),
                consumer_changes: PersistentHashMap::new_persistent_overlay(),
            },
            _custody: None,
        }))
    }

    pub(in super::super) fn maps(&self) -> &ForkSubscriptionMaps {
        self.0.maps()
    }

    pub(in super::super) fn detach(&mut self, custody: Option<Custody>) {
        if Arc::get_mut(&mut self.0).is_none() {
            self.0 = Arc::new(Owner {
                maps: self.maps().clone(),
                _custody: custody,
            });
        }
    }

    pub(in super::super) fn maps_mut(&mut self) -> &mut ForkSubscriptionMaps {
        Arc::get_mut(&mut self.0)
            .expect("publication detaches the subscriber overlay before editing")
            .maps_mut()
    }
}

impl std::fmt::Debug for ForkSubscriptions {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.maps().fmt(formatter)
    }
}

impl PartialEq for ForkSubscriptions {
    fn eq(&self, other: &Self) -> bool {
        self.maps() == other.maps()
    }
}

impl Eq for ForkSubscriptions {}

impl RetainedStorageMeasurement for ForkSubscriptions {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let ForkSubscriptionMaps {
            base,
            bucket_changes,
            consumer_changes,
        } = self.maps();
        Self::allocation_charge()?
            .checked_add(base.retained_heap_charge(work)?)?
            .checked_add(bucket_changes.retained_heap_charge(work)?)?
            .checked_add(consumer_changes.retained_heap_charge(work)?)
    }
}
