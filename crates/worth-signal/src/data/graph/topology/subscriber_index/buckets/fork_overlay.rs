use std::collections::BTreeMap;

use crate::data::handle::NodeId;

use super::{
    ForkConsumerMembershipChange, IndexedSubscriptionMembership, ProducerAspectKey,
    SubscriberScopeBuckets,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ReverseSubscriptionFlat {
    pub(super) buckets: BTreeMap<ProducerAspectKey, SubscriberScopeBuckets>,
    pub(super) by_consumer: BTreeMap<NodeId, Vec<IndexedSubscriptionMembership>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReverseSubscriptionStorage {
    Exclusive(ReverseSubscriptionFlat),
    ForkShared(ForkSubscriptions),
}

mod backing;
pub(super) use backing::{ForkSubscriptionMaps, ForkSubscriptions};

mod bucket_delta;
pub(super) use bucket_delta::BucketDelta;

mod set_delta;
pub(super) use set_delta::{SetDelta, SetMergeTraversal};

mod retained_charge;

mod merge;
pub(super) use merge::extend_merged_set;
