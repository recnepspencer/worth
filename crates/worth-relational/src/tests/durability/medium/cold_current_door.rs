//! The same current operation adapters serve successful and faulting tables.
use super::*;
use crate::storage::data::RelationalStorageUnavailable;
use crate::transactions::data::{
    CommitPreparationReason, DeleteRelationIntent, EntityMutationIntent, RelationMutationIntent,
    TransactionCommitError, UpdateEntityFieldsIntent,
};

#[derive(Debug, Clone, Copy)]
pub(super) enum CurrentDoor {
    Projection(super::projection_point_door::ProjectionPointDoor),
    Entity,
    Relation,
    Validation,
    Candidate,
    Commit,
    DeleteRelation,
}
impl CurrentDoor {
    pub(super) const ALL: [Self; 18] = [
        Self::Projection(super::projection_point_door::ProjectionPointDoor::EntityRecord),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::EntityRecordAdmitted),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::EntityRevision),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::EntityRevisionAdmitted),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::RelationRevision),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::AdjacencyIds),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::EntityState),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::RelationMetadata),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::AspectVersion),
        Self::Projection(super::projection_point_door::ProjectionPointDoor::StructuralRevision),
        Self::Projection(
            super::projection_point_door::ProjectionPointDoor::StructuralRevisionAdmitted,
        ),
        Self::Projection(
            super::projection_point_door::ProjectionPointDoor::BoundedStructuralRevision,
        ),
        Self::Entity,
        Self::Relation,
        Self::Validation,
        Self::Candidate,
        Self::Commit,
        Self::DeleteRelation,
    ];
    pub(super) fn publishes(self) -> bool {
        matches!(self, Self::Commit | Self::DeleteRelation)
    }
    pub(super) fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
        entity: EntityId,
        relation: RelationId,
    ) -> Result<(), RelationalStorageUnavailable> {
        let mut transaction = test_owner_begin_transaction_for_main(runtime);
        match self {
            Self::Projection(door) => {
                door.run(execution, runtime, entity, relation)
                    .map_err(|denial| match denial {
                        crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
                            failure,
                        ) => failure,
                        other => {
                            panic!("current projection preserves its storage failure: {other:?}")
                        }
                    })?
            }
            Self::Entity => assert_eq!(
                transaction
                    .read_entity(execution, entity)
                    .map_err(read_storage)?
                    .effective()
                    .unwrap()
                    .entity_id,
                entity
            ),
            Self::Relation => assert_eq!(
                transaction
                    .read_relation(execution, relation)
                    .map_err(read_storage)?
                    .effective()
                    .unwrap()
                    .relation_id,
                relation
            ),
            Self::DeleteRelation => {
                let mut deletion = test_owner_begin_transaction_for_main(runtime);
                deletion
                    .push_batch(
                        WorkerIntentBatch::new("head-only-delete").push(MutationIntent::Relation(
                            RelationMutationIntent::Delete(DeleteRelationIntent {
                                relation_id: relation,
                            }),
                        )),
                        execution.allocation_policy(),
                    )
                    .unwrap();
                let outcome = deletion
                    .commit(execution, runtime)
                    .map_err(commit_storage)?;
                release_test_commit_snapshot(runtime, &outcome);
                assert_eq!(
                    transaction
                        .read_relation(execution, relation)
                        .map_err(read_storage)?
                        .effective()
                        .unwrap()
                        .retired_at_version,
                    None,
                    "the transaction retained the pre-delete observation"
                );
                let mut current = test_owner_begin_transaction_for_main(runtime);
                assert_eq!(
                    current
                        .read_relation(execution, relation)
                        .map_err(read_storage)?
                        .effective()
                        .unwrap()
                        .retired_at_version,
                    Some(runtime.current_version_id())
                );
            }
            Self::Validation | Self::Candidate | Self::Commit => {
                transaction
                    .push_batch(
                        WorkerIntentBatch::new("head-only-current-write").push(
                            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                                UpdateEntityFieldsIntent {
                                    entity_id: entity,
                                    fields: name_field_patch("head-only-after"),
                                },
                            )),
                        ),
                        execution.allocation_policy(),
                    )
                    .unwrap();
                match self {
                    Self::Validation => {
                        transaction
                            .validate(execution, runtime)
                            .map_err(commit_storage)?;
                    }
                    Self::Candidate => {
                        runtime
                            .prepare_branch_transaction(execution, transaction)
                            .map_err(commit_storage)?;
                    }
                    Self::Commit => {
                        let outcome = transaction
                            .commit(execution, runtime)
                            .map_err(commit_storage)?;
                        release_test_commit_snapshot(runtime, &outcome);
                    }
                    _ => unreachable!("selected current write door"),
                }
            }
        }
        Ok(())
    }
}
fn read_storage(
    denial: crate::mvcc::RelationalTransactionReadDenial,
) -> RelationalStorageUnavailable {
    match denial {
        crate::mvcc::RelationalTransactionReadDenial::StorageUnavailable(failure) => failure,
        other => panic!("current read must retain native storage refusal: {other:?}"),
    }
}
fn commit_storage(denial: TransactionCommitError) -> RelationalStorageUnavailable {
    match denial {
        TransactionCommitError::Preparation { error, .. } => match error.reason() {
            CommitPreparationReason::StorageUnavailable(failure) => failure,
            other => panic!("current preparation must retain native storage refusal: {other:?}"),
        },
        other => panic!("current storage refusal must be pre-effect: {other:?}"),
    }
}
