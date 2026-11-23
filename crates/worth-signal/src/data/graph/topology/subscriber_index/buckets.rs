mod fork_growth;
use std::collections::{BTreeMap, BTreeSet};

use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::{InternedPartitionSubscription, InternedScopePath, ScopeCoverage};

mod capacity;
mod consumer_payload;
pub(crate) use capacity::{candidate_map_memory_requirement, CandidateEpochBasis};
mod batch;
mod discovery;
mod flat_mutation;
#[cfg(test)]
mod fork_cost_tests;
#[cfg(test)]
mod fork_granule_tests;
mod fork_overlay;
#[cfg(test)]
mod leased_queries_tests;
#[cfg(test)]
mod model_tests;
mod operational_clone;
mod prepared;
pub(crate) use prepared::{PreparedCandidateEpoch, PreparedCandidateQueries};
mod persistent_fork;
mod query;

use consumer_payload::{ForkConsumerMembershipChange, ForkConsumerMemberships};
use flat_mutation::{insert_flat_membership, remove_flat_consumer};
use fork_overlay::{
    ForkSubscriptionMaps, ForkSubscriptions, ReverseSubscriptionFlat, ReverseSubscriptionStorage,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IndexedSubscriptionScope {
    Unscoped,
    Path(InternedScopePath, ScopeCoverage),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ProducerAspectKey {
    producer: NodeId,
    aspect: Aspect,
}

impl ProducerAspectKey {
    fn from_authoritative_edge(producer: NodeId, aspect: Aspect) -> Self {
        Self { producer, aspect }
    }

    fn from_committed_output(producer: NodeId, aspect: Aspect) -> Self {
        Self { producer, aspect }
    }
}

impl SignalGraph {
    pub(crate) fn indexed_consumers_for_declared_outputs(
        &self,
        producer: NodeId,
    ) -> Result<Vec<NodeId>, SignalError> {
        if !self.topology.reverse_subscriptions.is_valid() {
            return Err(SignalError::internal(
                "reverse subscription index requires authority rebuild",
            ));
        }
        let produces = self.get_contract(producer)?.semantics.produces;
        let mut consumers = Vec::new();
        for index in 0..crate::data::aspect::MAX_ASPECTS {
            let aspect = Aspect::new(index as u8);
            if !produces.contains(crate::data::aspect::AspectMask::from_aspect(aspect)) {
                continue;
            }
            consumers.extend(
                self.topology
                    .reverse_subscriptions
                    .query_whole_aspect(
                        producer,
                        aspect,
                        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
                    )?
                    .candidates,
            );
        }
        consumers.sort_unstable();
        consumers.dedup();
        consumers.retain(|consumer| self.is_alive(*consumer));
        Ok(consumers)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndexedSubscriptionMembership {
    key: ProducerAspectKey,
    pub(crate) scope: IndexedSubscriptionScope,
}

impl IndexedSubscriptionMembership {
    pub(crate) fn from_edge(
        producer: NodeId,
        aspect: Aspect,
        scope: Option<InternedPartitionSubscription>,
    ) -> Option<Self> {
        let scope = match scope {
            None => IndexedSubscriptionScope::Unscoped,
            Some(scope) => IndexedSubscriptionScope::Path(scope.path(), scope.coverage()),
        };
        Some(Self {
            key: ProducerAspectKey::from_authoritative_edge(producer, aspect),
            scope,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SubscriberScopeBuckets {
    all: BTreeSet<NodeId>,
    unscoped: BTreeSet<NodeId>,
    same_path: BTreeMap<InternedScopePath, BTreeSet<NodeId>>,
    subtree_covering: BTreeMap<InternedScopePath, BTreeSet<NodeId>>,
    subtree_members: BTreeMap<InternedScopePath, BTreeSet<NodeId>>,
}

#[derive(Debug)]
pub(crate) struct ReverseSubscriptionIndex {
    storage: ReverseSubscriptionStorage,
    valid: bool,
}

impl Clone for ReverseSubscriptionIndex {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            valid: self.valid,
        }
    }
}

impl Default for ReverseSubscriptionIndex {
    fn default() -> Self {
        Self {
            storage: ReverseSubscriptionStorage::Exclusive(ReverseSubscriptionFlat::default()),
            valid: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReverseSubscriptionQuery {
    pub(crate) candidates: Vec<NodeId>,
    pub(crate) bucket_probes: u64,
}

impl ReverseSubscriptionIndex {
    pub(super) fn mark_rebuilt(&mut self) {
        self.valid = true;
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.valid
    }

    pub(super) fn clear(&mut self) {
        self.storage = ReverseSubscriptionStorage::Exclusive(ReverseSubscriptionFlat::default());
        self.valid = false;
    }

    pub(super) fn fork_edit_charge(
        &self,
    ) -> Result<
        Option<crate::data::retained_storage::RetainedStorageCharge>,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        match self.storage {
            ReverseSubscriptionStorage::Exclusive(_) => Ok(None),
            ReverseSubscriptionStorage::ForkShared(_) => {
                ForkSubscriptions::detached_payload_charge().map(Some)
            }
        }
    }

    pub(super) fn publish_replacement(
        &mut self,
        consumer: NodeId,
        memberships: Vec<IndexedSubscriptionMembership>,
        custody: Option<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    ) {
        if let ReverseSubscriptionStorage::ForkShared(overlay) = &mut self.storage {
            overlay.detach(custody);
        }
        self.remove_consumer(consumer);
        for membership in &memberships {
            self.insert_membership(consumer, membership);
        }
        if !memberships.is_empty() {
            match &mut self.storage {
                ReverseSubscriptionStorage::Exclusive(flat) => {
                    flat.by_consumer.insert(consumer, memberships);
                }
                ReverseSubscriptionStorage::ForkShared(overlay) => {
                    overlay.maps_mut().consumer_changes.insert(
                        consumer,
                        ForkConsumerMembershipChange::Replaced(
                            ForkConsumerMemberships::from_owned(memberships),
                        ),
                    );
                }
            }
        }
    }

    #[cfg(test)]
    pub(super) fn replace_consumer(
        &mut self,
        consumer: NodeId,
        memberships: Vec<IndexedSubscriptionMembership>,
    ) {
        self.publish_replacement(consumer, memberships, None);
    }

    #[cfg(test)]
    pub(super) fn inject_candidate_drift_for_test(
        &mut self,
        producer: NodeId,
        aspect: Aspect,
        consumer: NodeId,
    ) {
        let membership = IndexedSubscriptionMembership::from_edge(producer, aspect, None)
            .expect("unscoped test membership must be indexable");
        if let ReverseSubscriptionStorage::ForkShared(overlay) = &mut self.storage {
            overlay.detach(None);
        }
        self.insert_membership(consumer, &membership);
    }

    fn insert_membership(&mut self, consumer: NodeId, membership: &IndexedSubscriptionMembership) {
        match &mut self.storage {
            ReverseSubscriptionStorage::Exclusive(flat) => {
                insert_flat_membership(flat, consumer, membership);
            }
            ReverseSubscriptionStorage::ForkShared(overlay) => {
                let ForkSubscriptionMaps {
                    base,
                    bucket_changes,
                    ..
                } = overlay.maps_mut();
                let base_bucket = base.buckets.get(&membership.key);
                let mut delta = bucket_changes
                    .get(&membership.key)
                    .cloned()
                    .unwrap_or_default();
                delta.insert(base_bucket, consumer, &membership.scope);
                if delta.is_empty() {
                    bucket_changes.remove(&membership.key);
                } else {
                    bucket_changes.insert(membership.key, delta);
                }
            }
        }
    }

    fn remove_consumer(&mut self, consumer: NodeId) {
        match &mut self.storage {
            ReverseSubscriptionStorage::Exclusive(flat) => remove_flat_consumer(flat, consumer),
            ReverseSubscriptionStorage::ForkShared(overlay) => {
                let ForkSubscriptionMaps {
                    base,
                    bucket_changes,
                    consumer_changes,
                } = overlay.maps_mut();
                let memberships = match consumer_changes.get(&consumer) {
                    Some(ForkConsumerMembershipChange::Replaced(memberships)) => {
                        Some(memberships.as_slice().to_vec())
                    }
                    Some(ForkConsumerMembershipChange::Removed) => None,
                    None => base.by_consumer.get(&consumer).cloned(),
                };
                let Some(memberships) = memberships else {
                    return;
                };
                if base.by_consumer.contains_key(&consumer) {
                    consumer_changes.insert(consumer, ForkConsumerMembershipChange::Removed);
                } else {
                    consumer_changes.remove(&consumer);
                }
                for membership in memberships {
                    let base_bucket = base.buckets.get(&membership.key);
                    let mut delta = bucket_changes
                        .get(&membership.key)
                        .cloned()
                        .unwrap_or_default();
                    delta.remove(base_bucket, consumer, &membership.scope);
                    if delta.is_empty() {
                        bucket_changes.remove(&membership.key);
                    } else {
                        bucket_changes.insert(membership.key, delta);
                    }
                }
            }
        }
    }
}
mod retained_charge;

#[cfg(test)]
mod work_tests;
