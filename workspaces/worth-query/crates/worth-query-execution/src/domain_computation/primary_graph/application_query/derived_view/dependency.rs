use std::collections::BTreeSet;

use worth_foundational::facade::{AspectFieldLocator, AspectKey};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalAdjacencyDirection,
};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ViewDependency {
    Entity(EntityId),
    Aspect(EntityId, AspectKey),
    Field(EntityId, AspectFieldLocator),
    Adjacency(EntityId, KindId, u8),
}

impl ViewDependency {
    pub(super) fn retained_bytes(&self) -> usize {
        let owned = match self {
            Self::Aspect(_, aspect) => aspect.owned_allocation_capacity_bytes(),
            Self::Field(_, locator) => locator.owned_allocation_capacity_bytes(),
            Self::Entity(_) | Self::Adjacency(..) => 0,
        };
        std::mem::size_of::<Self>()
            .saturating_add(4 * std::mem::size_of::<usize>())
            .saturating_add(owned)
    }
}

pub(super) fn source_dependencies(
    facts: &[WorthQueryApplicationObservedFact],
) -> Option<BTreeSet<ViewDependency>> {
    let mut dependencies = BTreeSet::new();
    for fact in facts {
        match fact {
            WorthQueryApplicationObservedFact::SourceEntity { entity_id } => {
                dependencies.insert(ViewDependency::Entity(*entity_id));
            }
            WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id, aspect, ..
            } => {
                dependencies.insert(ViewDependency::Aspect(*entity_id, aspect.clone()));
            }
            WorthQueryApplicationObservedFact::SourceFieldRevision {
                entity_id, locator, ..
            } => {
                dependencies.insert(ViewDependency::Field(*entity_id, locator.clone()));
            }
            WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                anchor,
                relation_kind,
                direction,
                ..
            } => {
                let direction = match direction {
                    RelationalAdjacencyDirection::Outgoing => 0,
                    RelationalAdjacencyDirection::Incoming => 1,
                };
                dependencies.insert(ViewDependency::Adjacency(
                    *anchor,
                    *relation_kind,
                    direction,
                ));
            }
            _ => return None,
        }
    }
    (!dependencies.is_empty()).then_some(dependencies)
}
