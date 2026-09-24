use worth_foundational::facade::{AspectFieldLocator, AspectKey, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::indexes::DerivedIndexId;

mod adjacency;
mod entity_touch;
mod indexed_entity_selection;
mod locator_identity;
mod source_currentness;
mod workflow_definition_predecessor;
mod workflow_instance_capacity;
mod workflow_transition_capacity;
pub(in crate::domain_computation::primary_graph) use adjacency::observe_adjacency;
pub(in crate::domain_computation::primary_graph) use indexed_entity_selection::observe_indexed_entity_selection;
pub(in crate::domain_computation::primary_graph) use source_currentness::WorthQuerySourceCurrentnessFailure;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
    SourceEntity {
        entity_id: EntityId,
    },
    SourceAspectRevision {
        entity_id: EntityId,
        aspect: AspectKey,
        native_revision: Option<u64>,
    },
    SourceAdjacencyRevision {
        relation_kind: KindId,
        anchor: EntityId,
        direction: worth_relational::facade::runtime::RelationalAdjacencyDirection,
        native_revision: Option<worth_relational::facade::identity::VersionId>,
        comparison_work_limit: usize,
        endpoints: Vec<EntityId>,
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
    /// Exact bounded transition occupancy after retained-idempotency recovery.
    WorkflowTransitionCapacity {
        relation_kind: KindId,
        instance: EntityId,
        maximum_transitions: usize,
        transitions: Vec<WorthQueryApplicationObservedRelation>,
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

    pub(crate) fn remains_equal_in(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        match self {
            Self::SourceEntity { entity_id } => runtime
                .read_truth()
                .project_snapshot(snapshot)
                .is_some_and(|view| {
                    view.entity_record_with_projection_scope(
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some(
                                record.lifecycle()
                                    == worth_relational::facade::storage::RecordLifecycleState::Live,
                            )
                        },
                    ) == Some(true)
                }),
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => runtime
                .read_truth()
                .project_snapshot(snapshot)
                .and_then(|view| view.entity_aspect_version(*entity_id, aspect))
                == Some(*native_revision),
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                native_revision,
                comparison_work_limit,
                ..
            } => runtime
                .read_truth()
                .project_snapshot(snapshot)
                .and_then(|view| {
                    view.bounded_adjacency_structural_revision(
                        *anchor,
                        *relation_kind,
                        *direction,
                        *comparison_work_limit,
                    )
                    .ok()
                })
                .is_some_and(|current| current.revision() == *native_revision),
            Self::Entity {
                entity_id, kind, ..
            } => runtime
                .read_truth()
                .project_snapshot(snapshot)
                .and_then(|view| {
                    view.entity_record_with_projection_scope(
                        *entity_id,
                        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
                        |record| {
                            Some((
                                record.kind_id(),
                                record.lifecycle()
                                    == worth_relational::facade::storage::RecordLifecycleState::Live,
                            ))
                        },
                    )
                })
                .is_some_and(|(current_kind, live)| current_kind == *kind && live),
            Self::Field {
                entity_id,
                kind,
                locator,
                value,
                ..
            } => super::observation::observe_field_value(
                runtime, snapshot, *entity_id, *kind, locator,
            )
            .is_some_and(|current| current == *value),
            Self::AbsentField {
                entity_id,
                kind,
                locator,
            } => matches!(
                super::observation::observe_field(runtime, snapshot, *entity_id, *kind, locator),
                Some(super::observation::WorthQueryApplicationFieldObservation::Absent)
            ),
            Self::Relation {
                relation_kind,
                from,
                to,
                matching_relations,
                ..
            } => super::observation::exact_relations(
                runtime,
                snapshot,
                *relation_kind,
                *from,
                *to,
            )
            .is_ok_and(|current| current == *matching_relations),
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                maximum_work_units,
                relations,
                ..
            } => adjacency::remains_equal(
                runtime,
                snapshot,
                *relation_kind,
                *anchor,
                *direction,
                *maximum_work_units,
                relations,
            ),
            Self::IndexedEntitySelection {
                index_id,
                entity_kind,
                locator,
                value,
                candidate_limit,
                candidates,
            } => indexed_entity_selection::remains_equal(
                runtime,
                snapshot,
                *index_id,
                *entity_kind,
                locator,
                value,
                *candidate_limit,
                candidates,
            ),
            Self::WorkflowDefinitionPredecessor {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => workflow_definition_predecessor::remains_equal(
                runtime,
                snapshot,
                *relation_kind,
                *lineage,
                *expected_definition,
                *maximum_work_units,
            ),
            Self::WorkflowDefinitionCurrent {
                relation_kind,
                lineage,
                expected_definition,
                maximum_work_units,
            } => workflow_definition_predecessor::remains_equal(
                runtime,
                snapshot,
                *relation_kind,
                Some(*lineage),
                Some(*expected_definition),
                *maximum_work_units,
            ),
            Self::WorkflowInstanceCapacity {
                relation_kind,
                lineage,
                maximum_instances,
                instances,
            } => workflow_instance_capacity::remains_equal(
                runtime,
                snapshot,
                *relation_kind,
                *lineage,
                *maximum_instances,
                instances,
            ),
            Self::WorkflowTransitionCapacity {
                relation_kind,
                instance,
                maximum_transitions,
                transitions,
            } => workflow_transition_capacity::remains_equal(
                runtime,
                snapshot,
                *relation_kind,
                *instance,
                *maximum_transitions,
                transitions,
            ),
        }
    }
}
