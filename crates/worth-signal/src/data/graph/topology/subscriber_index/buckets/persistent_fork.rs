use super::fork_overlay::BucketDelta;
use super::{ForkConsumerMembershipChange, NodeId, ProducerAspectKey, ReverseSubscriptionFlat};
use crate::data::persistent_hash_map::PersistentHashMap;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageBacking, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

use super::fork_overlay::ReverseSubscriptionStorage;
use super::ReverseSubscriptionIndex;

impl ReverseSubscriptionIndex {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        self.fork_with_resources(None)
    }
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        self.fork_with_resources(Some(resources))
    }
    fn fork_with_resources(
        &mut self,
        resources: Option<&mut SignalConditionalRetentionReservation>,
    ) -> Self {
        if let ReverseSubscriptionStorage::Exclusive(flat) = &mut self.storage {
            let custody = resources.map(|resources| {
                let growth =
                    arc_allocation_charge::<RetainedStorageBacking<ReverseSubscriptionFlat>>()
                        .and_then(|charge| {
                            charge.checked_add(PersistentHashMap::<
                                ProducerAspectKey,
                                BucketDelta,
                            >::empty_persistent_overlay_charge()?)
                        })
                        .and_then(|charge| {
                            charge.checked_add(PersistentHashMap::<
                                NodeId,
                                ForkConsumerMembershipChange,
                            >::empty_persistent_overlay_charge()?)
                        })
                        .expect("fixed reverse-index conversion fits");
                resources
                    .split_embedded(growth)
                    .expect("complete reverse conversion was reserved")
            });
            self.storage = ReverseSubscriptionStorage::ForkShared {
                base: Arc::new(RetainedStorageBacking::new(std::mem::take(flat), custody)),
                bucket_changes: PersistentHashMap::new_persistent_overlay(),
                consumer_changes: PersistentHashMap::new_persistent_overlay(),
            };
        }
        self.fork_storage_identity_impl()
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        self.fork_storage_identity_impl()
    }

    fn fork_storage_identity_impl(&self) -> Self {
        let storage = match &self.storage {
            ReverseSubscriptionStorage::ForkShared {
                base,
                bucket_changes,
                consumer_changes,
            } => ReverseSubscriptionStorage::ForkShared {
                base: Arc::clone(base),
                bucket_changes: bucket_changes.clone(),
                consumer_changes: consumer_changes.clone(),
            },
            ReverseSubscriptionStorage::Exclusive(_) => {
                unreachable!("persistent fork must install shared storage")
            }
        };
        Self {
            storage,
            valid: self.valid,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        matches!(
            (&self.storage, &other.storage),
            (
                ReverseSubscriptionStorage::ForkShared {
                    base: left_base,
                    bucket_changes: left_buckets,
                    consumer_changes: left_consumers,
                },
                ReverseSubscriptionStorage::ForkShared {
                    base: right_base,
                    bucket_changes: right_buckets,
                    consumer_changes: right_consumers,
                },
            ) if Arc::ptr_eq(left_base, right_base)
                && left_buckets.ptr_eq(right_buckets)
                && left_consumers.ptr_eq(right_consumers)
        )
    }
}
