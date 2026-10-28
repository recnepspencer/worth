//! Semantic member coordinates; logical keys cannot become writable entities.
use crate::identity::data::EntityId;
use crate::projected_member_domain::ProjectedMemberDomainIdentity;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared immutable descriptive coordinates keep evaluator navigation from
/// copying semantic names. Allocation itself is admitted by the source owner.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ProjectedMemberCoordinate {
    pub domain: ProjectedMemberDomainIdentity,
    pub carrier: EntityId,
    pub case: String,
}

/// A complete graph-member coordinate. Projected identity includes the actual
/// carrier generation and semantic case; routing ordinals are never identity.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum NativeMemberKey {
    Entity(EntityId),
    Projected(Arc<ProjectedMemberCoordinate>),
}

impl From<EntityId> for NativeMemberKey {
    fn from(entity: EntityId) -> Self {
        Self::Entity(entity)
    }
}

impl NativeMemberKey {
    pub const fn entity(&self) -> Option<EntityId> {
        match self {
            Self::Entity(entity) => Some(*entity),
            Self::Projected(_) => None,
        }
    }

    /// The physical record whose lifecycle controls this member's existence.
    pub fn carrier(&self) -> EntityId {
        match self {
            Self::Entity(entity) => *entity,
            Self::Projected(member) => member.carrier,
        }
    }

    pub(crate) fn name_bytes(&self) -> Option<usize> {
        match self {
            Self::Entity(_) => Some(0),
            Self::Projected(member) => member.domain.name_bytes()?.checked_add(member.case.len()),
        }
    }

    /// Conservative B-tree comparison work for this semantic query key. The
    /// physical representation adds no name-byte cost to its existing policy.
    pub(crate) fn comparison_work(&self, count: usize) -> Option<usize> {
        let levels = (usize::BITS - count.max(1).leading_zeros()) as usize;
        self.name_bytes()?.checked_mul(11)?.checked_mul(levels)
    }

    pub(crate) fn owned_bytes(&self) -> Option<usize> {
        match self {
            Self::Entity(_) => Some(0),
            Self::Projected(member) => member
                .domain
                .owned_bytes()
                .checked_add(member.case.capacity()),
        }
    }
}
