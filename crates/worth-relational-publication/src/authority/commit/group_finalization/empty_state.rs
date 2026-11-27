//! The group state of a scheme no commit has named yet: no live group, no
//! member and no retired name.
use std::collections::BTreeSet;

use contracts::aggregate_identity::transition::group::model::{
    AggregateGroupMembers, AggregateMemberRole,
};
use contracts::aggregate_identity::transition::group::state_view::{
    AggregateGroupHeader, AggregateGroupMembership, AggregateGroupStateView,
};
use contracts::lineage::data::naming::PersistentNameIdentity;
use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port as contracts;

pub(super) struct EmptyGroupState;

impl AggregateGroupStateView for EmptyGroupState {
    fn group(&self, _: &AggregateIdentity) -> Option<AggregateGroupHeader> {
        None
    }

    fn membership(&self, _: &PersistentNameIdentity) -> Option<AggregateGroupMembership> {
        None
    }

    fn is_retired(&self, _: &AggregateIdentity) -> bool {
        false
    }

    fn least_member(
        &self,
        _: &AggregateIdentity,
        _: Option<AggregateMemberRole>,
        _: &BTreeSet<PersistentNameIdentity>,
    ) -> Option<PersistentNameIdentity> {
        None
    }

    fn members(&self, _: &AggregateIdentity) -> AggregateGroupMembers {
        AggregateGroupMembers::new()
    }
}
