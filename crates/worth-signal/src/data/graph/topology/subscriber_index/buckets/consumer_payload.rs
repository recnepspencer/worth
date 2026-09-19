use std::sync::Arc;

use super::IndexedSubscriptionMembership;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ForkConsumerMemberships(Arc<[IndexedSubscriptionMembership]>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ForkConsumerMembershipChange {
    Removed,
    Replaced(ForkConsumerMemberships),
}

impl ForkConsumerMemberships {
    pub(super) fn from_owned(memberships: Vec<IndexedSubscriptionMembership>) -> Self {
        Self(memberships.into())
    }

    pub(super) fn as_slice(&self) -> &[IndexedSubscriptionMembership] {
        &self.0
    }

    pub(super) fn to_owned(&self) -> Vec<IndexedSubscriptionMembership> {
        self.0.to_vec()
    }
}
mod retained_charge;
