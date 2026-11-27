//! Allocation-independent group naming vocabulary: names, roles, continuity.
//!
//! Every group member enters by its persistent name. The native carrier rides
//! along only so storage can locate the member; it never selects an anchor,
//! a keeper, or a byte of an identity.
use std::collections::btree_map::{self, BTreeMap};

use worth_foundational::facade::AggregateIdentity;

use super::denial::AggregateGroupNamingDenial;
use crate::graph_member::NativeMemberKey;
use crate::lineage::data::PersistentNameIdentity;

/// Closed consumer-driven continuity rule for group split/merge keeper selection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum AggregateContinuityRule {
    /// Follows its anchor carrier (e.g., a loop follows its host face).
    Anchor,
    /// Follows its enclosing boundary members (e.g., a space follows its walls).
    Boundary,
    /// Follows its owning producer role (e.g., a shell follows its body feature).
    ProducerRole,
}

impl AggregateContinuityRule {
    /// Stable canonical byte hashed into every v2 group identity.
    pub const fn canonical_tag(self) -> u8 {
        match self {
            Self::Anchor => 1,
            Self::Boundary => 2,
            Self::ProducerRole => 3,
        }
    }
}

/// Role a member plays for its group's declared continuity rule.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum AggregateMemberRole {
    Ordinary,
    /// Bears the scheme's declared anchor (or producer) role.
    Anchor,
    /// Encloses the group for [`AggregateContinuityRule::Boundary`].
    Boundary,
}

/// Validated dotted group scheme identifier (e.g. `cad.Space.v1`).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AggregateGroupSchemeId(String);

impl AggregateGroupSchemeId {
    /// Accepts exactly the dotted grammar of Declaration's
    /// `validate_scheme_identifier`: nonempty, no whitespace or control
    /// characters, no leading/trailing `.`, and no empty `..` segment.
    pub fn new(id: impl Into<String>) -> Result<Self, AggregateGroupNamingDenial> {
        let id = id.into();
        let invalid = id.is_empty()
            || id.starts_with('.')
            || id.ends_with('.')
            || id.contains("..")
            || id.chars().any(|ch| ch.is_whitespace() || ch.is_control());
        if invalid {
            return Err(AggregateGroupNamingDenial::InvalidSchemeId { scheme_id: id });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Installed group scheme: one group kind and its declared continuity rule.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AggregateGroupScheme {
    id: AggregateGroupSchemeId,
    rule: AggregateContinuityRule,
}

impl AggregateGroupScheme {
    pub const fn new(id: AggregateGroupSchemeId, rule: AggregateContinuityRule) -> Self {
        Self { id, rule }
    }

    pub const fn id(&self) -> &AggregateGroupSchemeId {
        &self.id
    }

    pub const fn rule(&self) -> AggregateContinuityRule {
        self.rule
    }
}

/// One named member: its storage carrier and its continuity role.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AggregateGroupMember {
    pub carrier: NativeMemberKey,
    pub role: AggregateMemberRole,
}

/// A nonempty-checked member set keyed solely by persistent name.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct AggregateGroupMembers(BTreeMap<PersistentNameIdentity, AggregateGroupMember>);

impl AggregateGroupMembers {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one member; a repeated name is refused rather than overwritten.
    pub fn insert(
        &mut self,
        name: PersistentNameIdentity,
        carrier: NativeMemberKey,
        role: AggregateMemberRole,
    ) -> Result<(), AggregateGroupNamingDenial> {
        match self.0.entry(name) {
            btree_map::Entry::Occupied(_) => {
                Err(AggregateGroupNamingDenial::DuplicateMemberName { name })
            }
            btree_map::Entry::Vacant(slot) => {
                slot.insert(AggregateGroupMember { carrier, role });
                Ok(())
            }
        }
    }

    pub fn get(&self, name: &PersistentNameIdentity) -> Option<&AggregateGroupMember> {
        self.0.get(name)
    }

    /// Removes `name`, returning its member.
    pub fn remove(&mut self, name: &PersistentNameIdentity) -> Option<AggregateGroupMember> {
        self.0.remove(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Members in persistent-name order, never native allocation order.
    pub fn iter(&self) -> impl Iterator<Item = (&PersistentNameIdentity, &AggregateGroupMember)> {
        self.0.iter()
    }
}

/// A live group before this commit, carrying the birth anchor of its name.
///
/// Not `Clone`: it owns a complete member set, so a copy is an `O(M)` walk.
#[derive(Debug, Eq, PartialEq)]
pub struct AggregatePriorGroup {
    pub identity: AggregateIdentity,
    pub anchor: PersistentNameIdentity,
    /// Strictly ascending prior sources of a split/merge successor name;
    /// empty for a pure `(scheme, anchor, rule)` group name.
    pub sources: Vec<AggregateIdentity>,
    pub members: AggregateGroupMembers,
}

/// How a group in the next state obtained its identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum AggregateGroupOrigin {
    /// Continues a prior live group selected by the continuity rule.
    Kept,
    /// A fresh identity: a pure group name with no live prior source, or a
    /// split/merge successor qualified by its prior sources.
    Born,
    /// A retired identity re-formed around its original anchor.
    Revived,
}

/// One named group of the next state with its complete membership: the
/// full-partition authority's result, which the canonical
/// [`super::delta::AggregateGroupAssignment`] records as a member delta for a
/// kept group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupPartitionAssignment {
    pub identity: AggregateIdentity,
    pub anchor: PersistentNameIdentity,
    /// The prior sources hashed into this name (empty for a pure group name).
    pub sources: Vec<AggregateIdentity>,
    pub members: AggregateGroupMembers,
    pub origin: AggregateGroupOrigin,
}

/// One live group named by a collision refusal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AggregateGroupClaimant {
    /// The anchor whose name this group claims.
    pub anchor: PersistentNameIdentity,
    /// The group's least member name, which names the group itself.
    pub least_member: PersistentNameIdentity,
    /// The prior live identity this claimant holds: the group it keeps, or
    /// itself when the claimant is a prior group retiring in this commit.
    pub live_prior: Option<AggregateIdentity>,
}

/// Operations used by the Relational owners.
pub trait AggregateGroupMembersOwnerPort: Sized {
    fn least_name(&self) -> Option<PersistentNameIdentity>;
    fn least_with(&self, role: AggregateMemberRole) -> Option<PersistentNameIdentity>;
}

impl AggregateGroupMembersOwnerPort for AggregateGroupMembers {
    fn least_name(&self) -> Option<PersistentNameIdentity> {
        self.0.keys().next().copied()
    }

    fn least_with(&self, role: AggregateMemberRole) -> Option<PersistentNameIdentity> {
        self.0
            .iter()
            .find(|(_, member)| member.role == role)
            .map(|(name, _)| *name)
    }
}
