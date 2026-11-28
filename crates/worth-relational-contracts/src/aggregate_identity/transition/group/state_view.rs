//! The point and ordered reads of one scheme's live groups that the
//! incremental group naming authority performs.
//!
//! A branch's group state answers each read in `O(log M)` without
//! materialising any group's membership; only the explicitly `O(M)`
//! [`AggregateGroupStateView::members`] walks one group.
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use super::model::CanonicalGroupMemberIdentity;
use super::model::{AggregateGroupMember, AggregateGroupMembers, AggregateMemberRole};

/// A live group's name parts and member counts, without its members.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupHeader {
    pub identity: AggregateIdentity,
    pub anchor: CanonicalGroupMemberIdentity,
    /// Strictly ascending prior sources of a split/merge successor name.
    pub sources: Vec<AggregateIdentity>,
    /// Live members holding each role, indexed by [`AggregateGroupRoleCounts::slot`].
    pub counts: AggregateGroupRoleCounts,
}

/// Member counts of one live group by role.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct AggregateGroupRoleCounts([u64; 3]);

impl AggregateGroupRoleCounts {
    pub const fn slot(role: AggregateMemberRole) -> usize {
        match role {
            AggregateMemberRole::Ordinary => 0,
            AggregateMemberRole::Anchor => 1,
            AggregateMemberRole::Boundary => 2,
        }
    }

    pub const fn from_slots(slots: [u64; 3]) -> Self {
        Self(slots)
    }

    pub const fn slots(&self) -> [u64; 3] {
        self.0
    }

    pub const fn of(&self, role: AggregateMemberRole) -> u64 {
        self.0[Self::slot(role)]
    }

    /// Every live member.
    pub const fn total(&self) -> u64 {
        self.0[0]
            .saturating_add(self.0[1])
            .saturating_add(self.0[2])
    }

    /// Counts one more member holding `role`.
    pub fn add(&mut self, role: AggregateMemberRole) {
        let slot = &mut self.0[Self::slot(role)];
        *slot = slot.saturating_add(1);
    }

    /// Counts one fewer member holding `role`; `None` when none was counted.
    #[must_use]
    pub fn remove(&mut self, role: AggregateMemberRole) -> Option<()> {
        let slot = &mut self.0[Self::slot(role)];
        *slot = slot.checked_sub(1)?;
        Some(())
    }

    /// The counts of a complete member set.
    pub fn of_members(members: &AggregateGroupMembers) -> Self {
        let mut counts = Self::default();
        for (_, member) in members.iter() {
            counts.add(member.role);
        }
        counts
    }
}

/// The live group one member belongs to, and the member's place in it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupMembership {
    pub group: AggregateIdentity,
    pub member: AggregateGroupMember,
}

/// One installed scheme's live groups, read for one transition.
pub trait AggregateGroupStateView {
    /// The live group `identity`. Complexity: O(log G).
    fn group(&self, identity: &AggregateIdentity) -> Option<AggregateGroupHeader>;

    /// The live group `name` belongs to and its carrier and role there.
    /// Complexity: O(log M).
    fn membership(&self, name: &CanonicalGroupMemberIdentity) -> Option<AggregateGroupMembership>;

    /// Whether `identity` is a retired, unrevived group name. Complexity: O(log G).
    fn is_retired(&self, identity: &AggregateIdentity) -> bool;

    /// The least member of `group` holding `role` (any role when `None`)
    /// that is not in `skip`.
    ///
    /// Complexity: O((|skip| + 1) log M): an ordered seek that passes at
    /// most `|skip|` members.
    fn least_member(
        &self,
        group: &AggregateIdentity,
        role: Option<AggregateMemberRole>,
        skip: &BTreeSet<CanonicalGroupMemberIdentity>,
    ) -> Option<CanonicalGroupMemberIdentity>;

    /// Every member of `group`, ascending by name.
    ///
    /// Complexity: O(M): only a born, revived or dissolved group's members,
    /// or a history selection, are ever walked.
    fn members(&self, group: &AggregateIdentity) -> AggregateGroupMembers;
}
