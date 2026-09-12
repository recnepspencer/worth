use worth_foundational::facade::AspectKey;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};

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
}

impl WorthQueryApplicationObservedFact {
    pub(super) const fn observed_field_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Field { value, .. } => Some(value),
            _ => None,
        }
    }

    pub(crate) fn locator_identity(&self) -> String {
        match self {
            Self::SourceEntity { entity_id } => format!(
                "application-source-entity:{}:{}:{}",
                entity_id.partition_value(),
                entity_id.local_slot_value(),
                entity_id.generation_value()
            ),
            Self::SourceAspectRevision {
                entity_id, aspect, ..
            } => format!(
                "application-source-aspect:{}:{}:{}:{}",
                entity_id.partition_value(),
                entity_id.local_slot_value(),
                entity_id.generation_value(),
                aspect.as_str()
            ),
            Self::SourceAdjacencyRevision {
                relation_kind,
                anchor,
                direction,
                ..
            } => format!(
                "application-source-adjacency:{direction:?}:{}:{}:{}:kind:{}",
                anchor.partition_value(),
                anchor.local_slot_value(),
                anchor.generation_value(),
                relation_kind.as_u32()
            ),
            Self::Entity {
                entity_id, kind, ..
            } => format!(
                "application-entity:{}:{}:{}:kind:{}",
                entity_id.partition_value(),
                entity_id.local_slot_value(),
                entity_id.generation_value(),
                kind.as_u32()
            ),
            Self::Field {
                entity_id, locator, ..
            }
            | Self::AbsentField {
                entity_id, locator, ..
            } => format!(
                "application-field:{}:{}:{}:{}/{}",
                entity_id.partition_value(),
                entity_id.local_slot_value(),
                entity_id.generation_value(),
                locator.aspect().aspect_key().as_str(),
                locator
                    .field_path()
                    .fields()
                    .first()
                    .expect("installed application fields have one field path")
                    .as_str()
            ),
            Self::Relation {
                relation_kind,
                from,
                to,
                ..
            } => format!(
                "application-relation:{}:{}:{}->{}:{}:{}:kind:{}",
                from.partition_value(),
                from.local_slot_value(),
                from.generation_value(),
                to.partition_value(),
                to.local_slot_value(),
                to.generation_value(),
                relation_kind.as_u32()
            ),
            Self::Adjacency {
                relation_kind,
                anchor,
                direction,
                ..
            } => format!(
                "application-adjacency:{direction:?}:{}:{}:{}:kind:{}",
                anchor.partition_value(),
                anchor.local_slot_value(),
                anchor.generation_value(),
                relation_kind.as_u32()
            ),
        }
    }

    pub(super) fn touches_entity(&self, candidate: EntityId) -> bool {
        match self {
            Self::SourceEntity { entity_id } => *entity_id == candidate,
            Self::SourceAspectRevision { entity_id, .. } => *entity_id == candidate,
            Self::SourceAdjacencyRevision {
                anchor, endpoints, ..
            } => *anchor == candidate || endpoints.contains(&candidate),
            Self::Entity { entity_id, .. }
            | Self::Field { entity_id, .. }
            | Self::AbsentField { entity_id, .. } => *entity_id == candidate,
            Self::Relation { from, to, .. } => *from == candidate || *to == candidate,
            Self::Adjacency {
                anchor, relations, ..
            } => {
                *anchor == candidate
                    || relations
                        .iter()
                        .any(|relation| relation.from == candidate || relation.to == candidate)
            }
        }
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
            } => observe_adjacency(
                runtime,
                snapshot,
                *relation_kind,
                *anchor,
                *direction,
                *maximum_work_units,
            )
            .is_some_and(|current| current == *relations),
        }
    }
}

pub(super) fn observe_adjacency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation_kind: KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    maximum_work_units: usize,
) -> Option<Vec<WorthQueryApplicationObservedRelation>> {
    let read = match direction {
        WorthQueryApplicationAdjacencyDirection::Outgoing => runtime
            .read_truth()
            .bounded_outgoing_relations_of_kind_at_version(
                anchor,
                relation_kind,
                snapshot.version_id(),
                maximum_work_units,
            ),
        WorthQueryApplicationAdjacencyDirection::Incoming => runtime
            .read_truth()
            .bounded_incoming_relations_of_kind_at_version(
                anchor,
                relation_kind,
                snapshot.version_id(),
                maximum_work_units,
            ),
    }
    .ok()?;
    Some(
        read.into_records()
            .into_iter()
            .map(|record| WorthQueryApplicationObservedRelation {
                relation_id: record.relation_id,
                from: record.source,
                to: record.target,
            })
            .collect(),
    )
}
