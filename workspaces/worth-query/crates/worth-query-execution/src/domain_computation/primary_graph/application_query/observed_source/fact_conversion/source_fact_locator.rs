//! Native coordinates for deduplicating one observed source fact set.

use super::Fact;
use worth_foundational::facade::{AspectFieldLocator, AspectKey};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalAdjacencyDirection,
};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum SourceFactLocator {
    Entity(EntityId),
    Aspect(EntityId, AspectKey),
    Field(EntityId, AspectFieldLocator),
    OutgoingAdjacency(EntityId, KindId),
    IncomingAdjacency(EntityId, KindId),
}

impl SourceFactLocator {
    pub(super) fn from_fact(fact: &Fact) -> Self {
        match fact {
            Fact::SourceEntity { entity_id } => Self::Entity(*entity_id),
            Fact::SourceAspectRevision {
                entity_id, aspect, ..
            } => Self::Aspect(*entity_id, aspect.clone()),
            Fact::SourceFieldRevision {
                entity_id, locator, ..
            } => Self::Field(*entity_id, locator.clone()),
            Fact::SourceAdjacencyRevision {
                anchor,
                relation_kind,
                direction,
                ..
            } => match direction {
                RelationalAdjacencyDirection::Outgoing => {
                    Self::OutgoingAdjacency(*anchor, *relation_kind)
                }
                RelationalAdjacencyDirection::Incoming => {
                    Self::IncomingAdjacency(*anchor, *relation_kind)
                }
            },
            _ => unreachable!("source conversion emits only native source facts"),
        }
    }
}
