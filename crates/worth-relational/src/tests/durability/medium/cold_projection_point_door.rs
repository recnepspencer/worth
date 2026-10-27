//! Current projection points must admit Head before exposing callbacks or revisions.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::visibility::materialization::read_records::{
    ProjectionAspectScope, RelationalAdjacencyDirection, RelationalAdjacencyVisit,
};
use std::cell::Cell;
#[derive(Clone, Copy, Debug)]
pub(super) enum ProjectionPointDoor {
    EntityRecord,
    EntityRecordAdmitted,
    EntityRevision,
    EntityRevisionAdmitted,
    RelationRevision,
    AdjacencyIds,
    EntityState,
    RelationMetadata,
    AspectVersion,
    StructuralRevision,
    StructuralRevisionAdmitted,
    BoundedStructuralRevision,
}
impl ProjectionPointDoor {
    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        entity: EntityId,
        relation: RelationId,
    ) -> Result<(), RelationalExecutionDenialCause> {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let view = runtime
            .read_truth()
            .project_observation(&basis.observation())
            .unwrap();
        let callbacks = Cell::new(0);
        let admit = |_| {
            callbacks.set(callbacks.get() + 1);
            Ok::<_, std::convert::Infallible>(())
        };
        let locator = revision_locator("name", "name");
        let direction = RelationalAdjacencyDirection::Outgoing;
        let answer = match self {
            Self::EntityState => view
                .read_exact_entity_state(execution, entity, |metadata, _| {
                    callbacks.set(callbacks.get() + 1);
                    metadata.entity_id
                })
                .map_err(borrowed_denial)
                .map(|answer| assert_eq!(answer, Some(entity))),
            Self::RelationMetadata => view
                .read_exact_relation_metadata(execution, relation)
                .map_err(borrowed_denial)
                .map(|answer| assert_eq!(answer.unwrap().relation_id, relation)),
            Self::AspectVersion => view
                .read_entity_aspect_version(execution, entity, &aspect_key("name"))
                .map_err(borrowed_denial)
                .map(|answer| assert!(answer.unwrap().is_some())),
            Self::StructuralRevision => view
                .read_exact_adjacency_structural_revision(execution, entity, KindId(2), direction)
                .map_err(borrowed_denial)
                .map(|answer| assert!(answer.is_some())),
            Self::StructuralRevisionAdmitted => view
                .read_exact_adjacency_structural_revision_admitted(
                    execution,
                    entity,
                    KindId(2),
                    direction,
                    admit,
                )
                .unwrap()
                .map_err(borrowed_denial)
                .map(|answer| assert!(answer.is_some())),
            Self::BoundedStructuralRevision => view
                .bounded_adjacency_structural_revision(execution, entity, KindId(2), direction, 1)
                .map_err(|denial| match denial {
                    crate::runtime::AdjacencyStructuralRevisionDenial::Execution(cause) => cause,
                    other => panic!("real adjacency must be available: {other:?}"),
                })
                .map(|answer| assert!(answer.revision().is_some())),
            Self::EntityRecord => view
                .entity_record_with_projection_scope(
                    execution,
                    entity,
                    ProjectionAspectScope::empty(),
                    |_| {
                        callbacks.set(callbacks.get() + 1);
                        Some(true)
                    },
                )
                .map(|value| assert_eq!(value, Some(true))),
            Self::EntityRecordAdmitted => view
                .entity_record_with_projection_scope_admitted(
                    execution,
                    entity,
                    ProjectionAspectScope::empty(),
                    admit,
                    |_| Some(true),
                )
                .map(|value| assert_eq!(value.unwrap(), Some(true))),
            Self::EntityRevision => {
                view.entity_field_revision(execution, entity, &locator)
                    .map(|revision| {
                        assert!(revision.is_some(), "the producer installed the named field")
                    })
            }
            Self::EntityRevisionAdmitted => view
                .entity_field_revision_admitted(execution, entity, &locator, admit)
                .map(|revision| {
                    assert!(
                        revision.unwrap().is_some(),
                        "the producer installed the named field"
                    )
                }),
            Self::RelationRevision => view
                .relation_field_revision(execution, relation, &revision_locator("label", "label"))
                .map(|revision| {
                    assert!(
                        revision.is_some(),
                        "the producer installed the relation label"
                    )
                }),
            Self::AdjacencyIds => {
                let saw_relation = Cell::new(false);
                view.try_visit_adjacency_ids(
                    execution,
                    entity,
                    KindId(2),
                    RelationalAdjacencyDirection::Outgoing,
                    |visit| {
                        callbacks.set(callbacks.get() + 1);
                        if let RelationalAdjacencyVisit::Relation(id) = visit {
                            saw_relation.set(saw_relation.get() || id == relation);
                        }
                        Ok::<_, std::convert::Infallible>(())
                    },
                )
                .map(|answer| {
                    answer.unwrap();
                    assert!(saw_relation.get(), "the producer's real edge survives");
                })
            }
        };
        if answer.is_err() {
            assert_eq!(
                callbacks.get(),
                0,
                "a failed residency guard exposes no provisional callback: {self:?}"
            );
        }
        answer
    }
}

fn revision_locator(aspect: &str, field: &str) -> worth_foundational::facade::AspectFieldLocator {
    worth_foundational::facade::AspectFieldLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        aspect_key(aspect),
        worth_foundational::facade::CanonicalFieldPath::single(field_key(field)),
    )
}

fn borrowed_denial(
    denial: crate::runtime::RelationalBorrowedRecordReadDenial,
) -> RelationalExecutionDenialCause {
    match denial {
        crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => cause,
        other => panic!("the producer supplies an exact basis: {other:?}"),
    }
}
