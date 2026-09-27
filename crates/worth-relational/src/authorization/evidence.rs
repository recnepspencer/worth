use worth_foundational::facade::AspectFieldLocator;

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::snapshots::data::SnapshotHandle;

use super::{
    RelationalAuthorizationEffectTarget, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPlanDenial, RelationalAuthorizationTraversalDirection,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RelationalAuthorizationObservationIdentity(pub(crate) [u8; 32]);

impl RelationalAuthorizationObservationIdentity {
    pub(crate) const fn mint(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationObservationFreshness {
    Fresh,
    Stale,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationalAuthorizationObservationCounters {
    pub paths_evaluated: usize,
    pub adjacency_lists_read: usize,
    pub adjacency_edges_inspected: usize,
    pub relation_records_inspected: usize,
    pub entity_records_inspected: usize,
    pub predicate_fields_inspected: usize,
    pub relation_join_index_lookups: usize,
    pub relation_join_candidates_inspected: usize,
    pub maximum_frontier_width: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationPathObservation {
    matched: bool,
    witness: Option<RelationalAuthorizationPathWitness>,
    dependencies: RelationalAuthorizationPathDependencies,
    exhaustive: bool,
}

/// One complete branch that independently satisfied an authorization path.
///
/// Ordinals preserve the installed traversal order, allowing a consumer to
/// retain an exact graph fact without reconstructing it from flattened
/// dependency sets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationPathWitness {
    entities: Vec<EntityId>,
}

impl RelationalAuthorizationPathWitness {
    pub(crate) const fn new(entities: Vec<EntityId>) -> Self {
        Self { entities }
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn entity_at(&self, traversal_ordinal: usize) -> Option<EntityId> {
        self.entities.get(traversal_ordinal).copied()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationalAuthorizationAdjacencyDependency {
    entity: EntityId,
    relation_kind: KindId,
    direction: RelationalAuthorizationTraversalDirection,
}

impl RelationalAuthorizationAdjacencyDependency {
    pub(crate) const fn new(
        entity: EntityId,
        relation_kind: KindId,
        direction: RelationalAuthorizationTraversalDirection,
    ) -> Self {
        Self {
            entity,
            relation_kind,
            direction,
        }
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn relation_kind(&self) -> KindId {
        self.relation_kind
    }

    pub const fn direction(&self) -> RelationalAuthorizationTraversalDirection {
        self.direction
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RelationalAuthorizationPathDependencies {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) relations: Vec<RelationId>,
    pub(crate) adjacency_lists: Vec<RelationalAuthorizationAdjacencyDependency>,
    pub(crate) fields: Vec<(EntityId, AspectFieldLocator)>,
}

impl RelationalAuthorizationPathObservation {
    pub(crate) fn new(
        matched: bool,
        witness: Option<RelationalAuthorizationPathWitness>,
        dependencies: RelationalAuthorizationPathDependencies,
        exhaustive: bool,
    ) -> Self {
        Self {
            matched,
            witness,
            dependencies,
            exhaustive,
        }
    }

    pub const fn matched(&self) -> bool {
        self.matched
    }

    pub const fn witness(&self) -> Option<&RelationalAuthorizationPathWitness> {
        self.witness.as_ref()
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.dependencies.entities
    }

    pub fn relations(&self) -> &[RelationId] {
        &self.dependencies.relations
    }

    pub fn adjacency_lists(&self) -> &[RelationalAuthorizationAdjacencyDependency] {
        &self.dependencies.adjacency_lists
    }

    pub fn fields(&self) -> &[(EntityId, AspectFieldLocator)] {
        &self.dependencies.fields
    }

    pub const fn exhaustive(&self) -> bool {
        self.exhaustive
    }

    pub(crate) fn has_same_decision_and_witness(&self, other: &Self) -> bool {
        self.matched == other.matched && self.witness == other.witness
    }
}

/// Relational-owned evidence minted from one actual immutable snapshot read.
///
/// Private fields and the absence of `Clone`/deserialization prevent callers
/// from restamping descriptive touched identifiers into observation authority.
#[derive(Debug)]
pub struct RelationalAuthorizationObservationEvidence {
    plan: RelationalAuthorizationObservationPlan,
    observation_identity: RelationalAuthorizationObservationIdentity,
    paths: Vec<RelationalAuthorizationPathObservation>,
    counters: RelationalAuthorizationObservationCounters,
}

impl RelationalAuthorizationObservationEvidence {
    pub(crate) fn mint(
        plan: RelationalAuthorizationObservationPlan,
        observation_identity: RelationalAuthorizationObservationIdentity,
        paths: Vec<RelationalAuthorizationPathObservation>,
        counters: RelationalAuthorizationObservationCounters,
    ) -> Self {
        Self {
            plan,
            observation_identity,
            paths,
            counters,
        }
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        self.plan.snapshot()
    }

    pub const fn observation_identity(&self) -> RelationalAuthorizationObservationIdentity {
        self.observation_identity
    }

    pub const fn principal(&self) -> EntityId {
        self.plan.principal()
    }

    pub const fn scope(&self) -> EntityId {
        self.plan.scope()
    }

    pub fn paths(&self) -> &[RelationalAuthorizationPathObservation] {
        &self.paths
    }

    pub fn proposed_effects(&self) -> &[RelationalAuthorizationEffectTarget] {
        self.plan.proposed_effects()
    }

    pub const fn counters(&self) -> RelationalAuthorizationObservationCounters {
        self.counters
    }

    pub(crate) fn comparison_plan(
        &self,
        snapshot: SnapshotHandle,
    ) -> Result<RelationalAuthorizationObservationPlan, RelationalAuthorizationPlanDenial> {
        self.plan.comparison_at(snapshot)
    }
}
