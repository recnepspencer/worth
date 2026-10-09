use worth_foundational::facade::{AspectFieldLocator, AspectKey, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId};

mod adjacency;
mod dependency_key;
mod entity_touch;
mod equality;
mod indexed_entity_selection;
mod locator_identity;
mod movement;
mod source_currentness;
mod source_merge;
mod workflow_definition_predecessor;
mod workflow_history_basis;
mod workflow_instance_capacity;
pub(in crate::domain_computation::primary_graph) use adjacency::observe_adjacency;
pub(in crate::domain_computation::primary_graph) use adjacency::{
    observe_adjacency_checked, AdjacencyObservationDenial,
};
pub(in crate::domain_computation::primary_graph) use indexed_entity_selection::reobserve as reobserve_indexed_entity_selection;
pub(in crate::domain_computation::primary_graph) use indexed_entity_selection::IndexedSelectionReobserveDenial;
pub(in crate::domain_computation::primary_graph) use indexed_entity_selection::{
    observe_indexed_candidates, observe_indexed_entity_selection, WorthQueryIndexedSelectionRefusal,
};
pub(in crate::domain_computation::primary_graph) use movement::{
    FactMovement, Movement, ObservedRetained,
};
pub(in crate::domain_computation::primary_graph) use source_currentness::WorthQuerySourceCurrentnessFailure;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) enum WorthQueryApplicationAdjacencyDirection {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation) struct WorthQueryApplicationObservedRelation {
    pub(in crate::domain_computation::primary_graph) relation_id: RelationId,
    pub(in crate::domain_computation::primary_graph) from: EntityId,
    pub(in crate::domain_computation::primary_graph) to: EntityId,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryApplicationFactKey {
    Entity {
        entity: String,
        entity_id: EntityId,
    },
    Field {
        entity: String,
        entity_id: EntityId,
        locator: AspectFieldLocator,
    },
    Relation {
        relation: String,
        from: EntityId,
        to: EntityId,
    },
    Adjacency {
        relation: String,
        anchor: EntityId,
        direction: WorthQueryApplicationAdjacencyDirection,
        maximum_work_units: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryApplicationObservedFact {
    /// Postcondition of a committed, declared output retirement. The original
    /// read locator remains covered; the exact entity generation must stay retired.
    RetiredOutputEntity {
        entity_id: EntityId,
        kind: KindId,
        created_at: worth_relational::facade::identity::VersionId,
        deleted_at: worth_relational::facade::identity::VersionId,
        read_locator: String,
    },
    SourceEntity {
        entity_id: EntityId,
    },
    SourceAspectRevision {
        entity_id: EntityId,
        aspect: AspectKey,
        native_revision: Option<u64>,
    },
    SourceFieldRevision {
        entity_id: EntityId,
        locator: AspectFieldLocator,
        native_revision: Option<worth_relational::facade::runtime::RelationalFieldRevision>,
    },
    SourceAdjacencyRevision {
        relation_kind: KindId,
        anchor: EntityId,
        direction: worth_relational::facade::runtime::RelationalAdjacencyDirection,
        native_revision: Option<worth_relational::facade::identity::VersionId>,
        comparison_work_limit: usize,
        endpoints: super::WorthQueryApplicationSourceAdjacencyEndpoints,
    },
    Entity {
        entity_id: EntityId,
        kind: KindId,
    },
    Field {
        entity_id: EntityId,
        kind: KindId,
        locator: AspectFieldLocator,
        value: AspectValue,
    },
    AbsentField {
        entity_id: EntityId,
        kind: KindId,
        locator: AspectFieldLocator,
    },
    Relation {
        relation_kind: KindId,
        from: EntityId,
        to: EntityId,
        matching_relations: Vec<RelationId>,
    },
    Adjacency {
        relation_kind: KindId,
        anchor: EntityId,
        direction: WorthQueryApplicationAdjacencyDirection,
        maximum_work_units: usize,
        relations: Vec<WorthQueryApplicationObservedRelation>,
    },
    /// Exact result of a bounded equality-index selection.
    ///
    /// This is owner-issued for Query-native application programs whose
    /// correctness depends on both presence and absence. Re-executing the
    /// same bounded lookup during provider recomparison closes the
    /// check-then-create race without exposing raw index authority.
    IndexedEntitySelection {
        index_id: DerivedIndexId,
        definition: std::sync::Arc<DerivedIndexDefinition>,
        entity_kind: KindId,
        locator: AspectFieldLocator,
        value: AspectValue,
        candidate_limit: usize,
        candidates: Vec<EntityId>,
    },
    /// Exact predecessor intent for a workflow-definition publication.
    ///
    /// A false observation is retained deliberately: retained idempotency is
    /// resolved before fact recomparison, so an identical retry can recover
    /// its receipt while a new effect attempt is forced stale.
    WorkflowDefinitionPredecessor {
        relation_kind: KindId,
        lineage: Option<EntityId>,
        expected_definition: Option<EntityId>,
        maximum_work_units: usize,
    },
    /// Requires one exact performed definition to remain current at commit.
    ///
    /// This may deliberately be false at preparation so an exact retained
    /// replay can resolve before a superseded definition makes a fresh start
    /// stale.
    WorkflowDefinitionCurrent {
        relation_kind: KindId,
        lineage: EntityId,
        expected_definition: EntityId,
        maximum_work_units: usize,
    },
    /// Exact bounded live-instance occupancy for one workflow lineage.
    ///
    /// Equality closes concurrent-start races. Capacity is also checked on
    /// recomparison, so a full lineage can replay a retained start but cannot
    /// commit a new instance.
    WorkflowInstanceCapacity {
        relation_kind: KindId,
        lineage: EntityId,
        maximum_instances: usize,
        instances: Vec<WorthQueryApplicationObservedRelation>,
    },
    /// A budgeted history read at an exact immutable native basis. It carries
    /// currentness, not permission to address arbitrary entities in that basis.
    WorkflowHistoryBasis {
        instance: EntityId,
        maximum_transitions: usize,
        transition_count: usize,
        snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    },
}

impl WorthQueryApplicationObservedFact {
    pub(super) const fn observed_field_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Field { value, .. } => Some(value),
            _ => None,
        }
    }

    pub(crate) fn locator_identity(&self) -> String {
        locator_identity::encode(self)
    }

    pub(super) fn touches_entity(&self, candidate: EntityId) -> bool {
        entity_touch::evaluate(self, candidate)
    }
}

impl WorthQueryApplicationObservedFact {
    pub(in crate::domain_computation::primary_graph) fn write_dependency_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        locator_identity::write(self, output)
    }
}
