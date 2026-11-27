use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use super::delta::{AggregateGroupAssignment, AggregateGroupChange, AggregateGroupMemberDelta};
use super::denial::AggregateGroupNamingDenial;
use super::model::{AggregateGroupMembers, AggregateGroupPartitionAssignment};
use crate::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;

/// The canonical naming outcome of one group transition: what a commit
/// publishes, with every kept group recorded by its member delta.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupTransition {
    /// Next-state groups in identity order.
    pub assignments: Vec<AggregateGroupAssignment>,
    pub retired: BTreeSet<AggregateIdentity>,
    pub lineage: BTreeSet<Event>,
}

impl AggregateGroupTransition {
    /// Whether the transition changes nothing: no retirement, no lineage
    /// event and no kept group's membership changed.
    pub fn is_noop(&self) -> bool {
        self.retired.is_empty()
            && self.lineage.is_empty()
            && self.assignments.iter().all(|assignment| {
                matches!(&assignment.change, AggregateGroupChange::Kept(delta) if delta.is_empty())
            })
    }
}

/// The complete naming outcome of one full-partition group transition, with
/// every next-state group's complete membership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupPartitionTransition {
    /// Next-state groups in identity order.
    pub assignments: Vec<AggregateGroupPartitionAssignment>,
    pub retired: BTreeSet<AggregateIdentity>,
    pub lineage: BTreeSet<Event>,
}

/// The minimal delta taking `prior` to `next`.
///
/// Complexity: O(|prior| + |next|) by one merge walk.
pub fn member_delta(
    prior: &AggregateGroupMembers,
    next: &AggregateGroupMembers,
) -> Result<AggregateGroupMemberDelta, AggregateGroupNamingDenial> {
    let mut delta = AggregateGroupMemberDelta::new();
    for (name, _) in prior.iter() {
        if next.get(name).is_none() {
            delta.remove(*name)?;
        }
    }
    for (name, member) in next.iter() {
        if prior.get(name) != Some(member) {
            delta.upsert(*name, member.carrier.clone(), member.role)?;
        }
    }
    Ok(delta)
}
