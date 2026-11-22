//! Typed refusals of the allocation-independent group naming authority.
use worth_foundational::facade::AggregateIdentity;

use super::{AggregateContinuityRule, AggregateGroupClaimant};
use crate::lineage::data::PersistentNameIdentity;

/// Why a group naming transition was refused. Every refusal has zero effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AggregateGroupNamingDenial {
    /// The scheme identifier is empty or outside `[a-z0-9._-]`.
    InvalidSchemeId { scheme_id: String },
    /// A prior or proposed group has no members.
    EmptyGroup,
    /// One member name was supplied twice for one group.
    DuplicateMemberName { name: PersistentNameIdentity },
    /// Two distinct member names claim one native carrier.
    DuplicateCarrier { name: PersistentNameIdentity },
    /// One member name belongs to two groups of the same state.
    OverlappingMember { name: PersistentNameIdentity },
    /// Two prior live groups carry one identity.
    DuplicatePriorIdentity { identity: AggregateIdentity },
    /// A prior identity is not the v2 name of its recorded birth anchor.
    PriorIdentityMismatch { identity: AggregateIdentity },
    /// A prior live identity is also recorded as retired.
    RetiredIdentityLive { identity: AggregateIdentity },
    /// The continuity rule requires a role member the group does not have.
    MissingAnchorMember {
        rule: AggregateContinuityRule,
        least_member: PersistentNameIdentity,
    },
    /// Two simultaneous live groups claim one `(scheme, anchor, rule)` name.
    LiveNameCollision {
        identity: AggregateIdentity,
        first: AggregateGroupClaimant,
        second: AggregateGroupClaimant,
    },
}
