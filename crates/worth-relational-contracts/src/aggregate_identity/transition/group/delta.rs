//! The canonical committed form of a group transition: a kept group records
//! only the members it lost, gained or re-carried, never its untouched
//! members, so a `k`-member edit of an `M`-member group is `O(k)` on the wire
//! and in every consumer.
use std::collections::btree_map::{self, BTreeMap};
use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use super::denial::AggregateGroupNamingDenial;
use super::model::{
    AggregateGroupMember, AggregateGroupMembers, AggregateGroupOrigin, AggregateMemberRole,
};
use crate::graph_member::NativeMemberKey;
use crate::lineage::data::PersistentNameIdentity;

/// The membership change of one kept group: members that left it, and
/// members that joined it or changed carrier or role in it.
///
/// It carries no untouched member by construction: a minimal delta lists a
/// member only when the kept group's membership of it changed, and the
/// branch group state refuses any upsert that changes nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct AggregateGroupMemberDelta {
    removed: BTreeSet<PersistentNameIdentity>,
    upserted: BTreeMap<PersistentNameIdentity, AggregateGroupMember>,
}

impl AggregateGroupMemberDelta {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that `name` left the group.
    pub fn remove(
        &mut self,
        name: PersistentNameIdentity,
    ) -> Result<(), AggregateGroupNamingDenial> {
        if self.upserted.contains_key(&name) || !self.removed.insert(name) {
            return Err(AggregateGroupNamingDenial::DuplicateMemberName { name });
        }
        Ok(())
    }

    /// Records that `name` joined the group or now holds `carrier` and `role`.
    pub fn upsert(
        &mut self,
        name: PersistentNameIdentity,
        carrier: NativeMemberKey,
        role: AggregateMemberRole,
    ) -> Result<(), AggregateGroupNamingDenial> {
        if self.removed.contains(&name) {
            return Err(AggregateGroupNamingDenial::DuplicateMemberName { name });
        }
        match self.upserted.entry(name) {
            btree_map::Entry::Occupied(_) => {
                Err(AggregateGroupNamingDenial::DuplicateMemberName { name })
            }
            btree_map::Entry::Vacant(slot) => {
                slot.insert(AggregateGroupMember { carrier, role });
                Ok(())
            }
        }
    }

    /// Members that left, ascending.
    pub fn removed(&self) -> impl Iterator<Item = &PersistentNameIdentity> {
        self.removed.iter()
    }

    /// Members that joined or changed carrier or role, ascending.
    pub fn upserted(
        &self,
    ) -> impl Iterator<Item = (&PersistentNameIdentity, &AggregateGroupMember)> {
        self.upserted.iter()
    }

    pub fn removed_len(&self) -> usize {
        self.removed.len()
    }

    pub fn upserted_len(&self) -> usize {
        self.upserted.len()
    }

    /// Whether the kept group's membership is unchanged.
    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.upserted.is_empty()
    }
}

/// How one next-state group's membership is recorded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AggregateGroupChange {
    /// A kept group: only its member delta (`O(touched)`).
    Kept(AggregateGroupMemberDelta),
    /// A newly named group with its initial members.
    Born(AggregateGroupMembers),
    /// A retired name re-formed, with its initial members.
    Revived(AggregateGroupMembers),
}

/// One named group of the next state, as committed and published.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupAssignment {
    pub identity: AggregateIdentity,
    pub anchor: PersistentNameIdentity,
    /// The prior sources hashed into this name (empty for a pure group name).
    pub sources: Vec<AggregateIdentity>,
    pub change: AggregateGroupChange,
}

impl AggregateGroupAssignment {
    pub const fn origin(&self) -> AggregateGroupOrigin {
        match self.change {
            AggregateGroupChange::Kept(_) => AggregateGroupOrigin::Kept,
            AggregateGroupChange::Born(_) => AggregateGroupOrigin::Born,
            AggregateGroupChange::Revived(_) => AggregateGroupOrigin::Revived,
        }
    }

    /// Every member name this assignment changes: a kept group's removed and
    /// upserted members, or a born or revived group's initial members.
    ///
    /// Complexity: O(touched members).
    pub fn touched_members(&self) -> Box<dyn Iterator<Item = &PersistentNameIdentity> + '_> {
        match &self.change {
            AggregateGroupChange::Kept(delta) => Box::new(
                delta
                    .removed()
                    .chain(delta.upserted().map(|(name, _)| name)),
            ),
            AggregateGroupChange::Born(members) | AggregateGroupChange::Revived(members) => {
                Box::new(members.iter().map(|(name, _)| name))
            }
        }
    }
}
