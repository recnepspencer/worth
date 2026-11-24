use std::collections::BTreeSet;

use worth_foundational::facade::AspectFieldLocator;

use worth_relational_contracts::internal_port::identity::data::record_ids::{EntityId, RelationId};

use super::evidence::RelationalAuthorizationPathDependencies;
use super::RelationalAuthorizationAdjacencyDependency;

pub(super) struct RelationalAuthorizationDependencySets {
    pub(super) entities: BTreeSet<EntityId>,
    pub(super) relations: BTreeSet<RelationId>,
    pub(super) adjacencies: BTreeSet<RelationalAuthorizationAdjacencyDependency>,
    pub(super) fields: BTreeSet<(EntityId, AspectFieldLocator)>,
}

impl RelationalAuthorizationDependencySets {
    pub(super) fn new(principal: EntityId) -> Self {
        Self {
            entities: BTreeSet::from([principal]),
            relations: BTreeSet::new(),
            adjacencies: BTreeSet::new(),
            fields: BTreeSet::new(),
        }
    }

    pub(super) fn finish(self) -> RelationalAuthorizationPathDependencies {
        RelationalAuthorizationPathDependencies {
            entities: self.entities.into_iter().collect(),
            relations: self.relations.into_iter().collect(),
            adjacency_lists: self.adjacencies.into_iter().collect(),
            fields: self.fields.into_iter().collect(),
        }
    }
}
