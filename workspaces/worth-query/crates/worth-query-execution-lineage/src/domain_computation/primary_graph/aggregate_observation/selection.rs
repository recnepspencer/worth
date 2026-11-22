//! Native selection preserves the difference between a relation and a definition.
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::PinnedAggregateReadScope,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AggregateSelection {
    OrbitRelation(KindId),
    ComponentDefinition(AggregateDefinitionIdentity),
}

impl AggregateSelection {
    pub fn scope(self, member: EntityId) -> PinnedAggregateReadScope<'static> {
        match self {
            Self::OrbitRelation(relation) => PinnedAggregateReadScope::new(relation, member),
            Self::ComponentDefinition(definition) => {
                PinnedAggregateReadScope::for_definition(definition, member)
            }
        }
    }

    pub fn orbit_relation(self) -> Option<KindId> {
        match self {
            Self::OrbitRelation(relation) => Some(relation),
            Self::ComponentDefinition(_) => None,
        }
    }
}
