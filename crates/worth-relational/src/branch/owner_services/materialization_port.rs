use std::collections::BTreeSet;

use crate::branch::{
    AdmittedRelationalBranchBasis, PreparedRelationalMaterializationSuspension,
    PreparedRelationalRematerialization, RelationalEntityMaterialization,
    RelationalMaterializationCustody, RelationalMaterializationError,
    RelationalMaterializationRecord, RelationalMaterializationSuspensionCompletion,
    RelationalRelationMaterialization, RelationalRematerializationCompletion,
    RelationalRematerializationFailure,
};
use crate::mvcc::{RelationalMaterializationTransactionMode, RelationalTransactionIntent};
use crate::transactions::data::{
    MaterializationMutationIntent, MutationIntent, RematerializeEntityIntent,
    RematerializeRelationIntent, SuspendEntityMaterializationIntent,
    SuspendRelationMaterializationIntent, WorkerIntentBatch,
};

use super::owner_binding::RelationalOwnerServiceBinding;

#[derive(Debug, Clone)]
pub struct RelationalMaterializationPort {
    owner: RelationalOwnerServiceBinding,
}

impl RelationalMaterializationPort {
    pub(super) fn new(owner: RelationalOwnerServiceBinding) -> Self {
        Self { owner }
    }

    pub fn prepare_generated_materialization_suspension(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        generated_entities: &[crate::identity::data::EntityId],
    ) -> Result<PreparedRelationalMaterializationSuspension, RelationalMaterializationError> {
        let runtime = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalMaterializationError::OwnerUnavailable)?;
        let source_commit = basis
            .inner
            .root
            .commit_id()
            .ok_or(RelationalMaterializationError::SourceCommitMismatch)?;
        let records = source_generated_manifest(basis, generated_entities)?;
        let mut transaction = runtime
            .begin_branch_transaction(
                basis,
                RelationalTransactionIntent::materialization(
                    RelationalMaterializationTransactionMode::Suspend,
                ),
            )
            .map_err(RelationalMaterializationError::TransactionAdmission)?;
        transaction
            .push_batch(suspension_batch(&records))
            .map_err(RelationalMaterializationError::TransactionStaging)?;
        let candidate = runtime
            .prepare_branch_transaction(transaction)
            .map_err(RelationalMaterializationError::Commit)?;
        let transaction_id = candidate.transaction_id();
        Ok(PreparedRelationalMaterializationSuspension {
            candidate,
            completion: RelationalMaterializationSuspensionCompletion {
                transaction_id,
                branch_identity: basis.identity().clone(),
                source_commit,
                records,
            },
        })
    }

    #[cfg(test)]
    pub(super) fn suspend_generated_materialization(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        generated_entities: &[crate::identity::data::EntityId],
    ) -> Result<crate::branch::RelationalMaterializationSuspension, RelationalMaterializationError>
    {
        let prepared =
            self.prepare_generated_materialization_suspension(basis, generated_entities)?;
        let (candidate, completion) = prepared.into_parts();
        let runtime = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalMaterializationError::OwnerUnavailable)?;
        let commit = runtime
            .publish_prepared_candidate(candidate)
            .map_err(RelationalMaterializationError::Commit)?;
        completion.complete(commit)
    }

    pub fn prepare_generated_rematerialization(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        custody: RelationalMaterializationCustody,
        entities: Vec<RelationalEntityMaterialization>,
        relations: Vec<RelationalRelationMaterialization>,
    ) -> Result<PreparedRelationalRematerialization, RelationalRematerializationFailure> {
        match self.prepare_rematerialization_inner(basis, &custody, entities, relations) {
            Ok((candidate, invariant_evidence)) => {
                let transaction_id = candidate.transaction_id();
                Ok(PreparedRelationalRematerialization {
                    candidate,
                    completion: RelationalRematerializationCompletion {
                        transaction_id,
                        custody,
                    },
                    invariant_evidence,
                })
            }
            Err(error) => Err(RelationalRematerializationFailure { custody, error }),
        }
    }

    #[cfg(test)]
    pub(super) fn rematerialize_generated_materialization(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        custody: RelationalMaterializationCustody,
        entities: Vec<RelationalEntityMaterialization>,
        relations: Vec<RelationalRelationMaterialization>,
    ) -> Result<crate::transactions::data::CommitResult, RelationalRematerializationFailure> {
        let prepared =
            self.prepare_generated_rematerialization(basis, custody, entities, relations)?;
        let (candidate, completion, _invariant_evidence) = prepared.into_parts();
        let Some(runtime) = self.owner.admitted_runtime() else {
            return Err(RelationalRematerializationFailure {
                custody: completion.into_custody(),
                error: RelationalMaterializationError::OwnerUnavailable,
            });
        };
        match runtime.publish_prepared_candidate(candidate) {
            Ok(commit) => completion.complete(commit),
            Err(error) => Err(RelationalRematerializationFailure {
                custody: completion.into_custody(),
                error: RelationalMaterializationError::Commit(error),
            }),
        }
    }

    fn prepare_rematerialization_inner(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        custody: &RelationalMaterializationCustody,
        entities: Vec<RelationalEntityMaterialization>,
        relations: Vec<RelationalRelationMaterialization>,
    ) -> Result<
        (
            crate::mvcc::PreparedRelationalCommitCandidate,
            crate::mvcc::RelationalMutationInvariantEvidence,
        ),
        RelationalMaterializationError,
    > {
        let runtime = self
            .owner
            .admitted_runtime()
            .ok_or(RelationalMaterializationError::OwnerUnavailable)?;
        validate_custody_basis(basis, custody)?;
        validate_candidate_manifest(custody, &entities, &relations)?;
        let mut transaction = runtime
            .begin_branch_transaction(
                basis,
                RelationalTransactionIntent::materialization(
                    RelationalMaterializationTransactionMode::Rematerialize,
                ),
            )
            .map_err(RelationalMaterializationError::TransactionAdmission)?;
        transaction
            .push_batch(rematerialization_batch(entities, relations))
            .map_err(RelationalMaterializationError::TransactionStaging)?;
        let validated = transaction
            .validate(&runtime)
            .map_err(RelationalMaterializationError::Commit)?;
        let invariant_evidence = validated.invariant_evidence().clone();
        let candidate = runtime
            .prepare_validated_proposal(validated)
            .map_err(RelationalMaterializationError::Commit)?;
        Ok((candidate, invariant_evidence))
    }
}

fn source_generated_manifest(
    basis: &AdmittedRelationalBranchBasis,
    generated_entities: &[crate::identity::data::EntityId],
) -> Result<Vec<RelationalMaterializationRecord>, RelationalMaterializationError> {
    let root = &basis.inner.root;
    let generated = generated_entities.iter().copied().collect::<BTreeSet<_>>();
    if generated.is_empty() || generated.len() != generated_entities.len() {
        return Err(RelationalMaterializationError::SourceIsNotCompleteCreatePublication);
    }
    let mut unique = BTreeSet::new();
    let mut incident_relations = BTreeSet::new();
    for entity_id in &generated {
        unique.insert(live_record_manifest(
            root,
            &crate::transactions::data::RecordRef::Entity(*entity_id),
        )?);
        if let Some(partition) = root.partition_state(entity_id.partition_id) {
            let slot = entity_id.slot_index();
            for adjacency in [
                partition.adjacency.get(slot),
                partition.reverse_adjacency.get(slot),
            ]
            .into_iter()
            .flatten()
            {
                incident_relations.extend(adjacency.current_ids().iter().copied());
            }
        }
    }
    for relation_id in incident_relations {
        if let Some(record) = live_relation_manifest(root, relation_id)? {
            unique.insert(record);
        }
    }
    Ok(unique.into_iter().collect())
}

fn live_relation_manifest(
    root: &crate::branch::RelationalBranchRoot,
    relation_id: crate::identity::data::RelationId,
) -> Result<Option<RelationalMaterializationRecord>, RelationalMaterializationError> {
    let Some(slot) = root
        .partition_state(relation_id.partition_id)
        .and_then(|partition| partition.relation_arena.get(&relation_id))
        .filter(|slot| slot.is_live())
    else {
        return Ok(None);
    };
    let kind_id = slot
        .kind_id()
        .ok_or_else(|| missing_relation(relation_id))?;
    let endpoints = slot
        .extra()
        .endpoints
        .as_ref()
        .ok_or_else(|| missing_relation(relation_id))?;
    Ok(Some(RelationalMaterializationRecord::Relation {
        relation_id,
        kind_id,
        source: endpoints.source,
        target: endpoints.target,
    }))
}

fn live_record_manifest(
    root: &crate::branch::RelationalBranchRoot,
    record: &crate::transactions::data::RecordRef,
) -> Result<RelationalMaterializationRecord, RelationalMaterializationError> {
    match record {
        crate::transactions::data::RecordRef::Entity(entity_id) => {
            let slot = root
                .partition_state(entity_id.partition_id)
                .and_then(|partition| partition.entity_arena.get(entity_id));
            let kind_id = slot
                .filter(|slot| slot.is_live())
                .and_then(|slot| slot.kind_id())
                .ok_or_else(|| missing_entity(*entity_id))?;
            Ok(RelationalMaterializationRecord::Entity {
                entity_id: *entity_id,
                kind_id,
            })
        }
        crate::transactions::data::RecordRef::Relation(relation_id) => {
            let slot = root
                .partition_state(relation_id.partition_id)
                .and_then(|partition| partition.relation_arena.get(relation_id))
                .filter(|slot| slot.is_live())
                .ok_or_else(|| missing_relation(*relation_id))?;
            let kind_id = slot
                .kind_id()
                .ok_or_else(|| missing_relation(*relation_id))?;
            let endpoints = slot
                .extra()
                .endpoints
                .as_ref()
                .ok_or_else(|| missing_relation(*relation_id))?;
            Ok(RelationalMaterializationRecord::Relation {
                relation_id: *relation_id,
                kind_id,
                source: endpoints.source,
                target: endpoints.target,
            })
        }
    }
}

fn missing_entity(entity_id: crate::identity::data::EntityId) -> RelationalMaterializationError {
    RelationalMaterializationError::SourceRecordUnavailable(
        crate::transactions::data::RecordRef::Entity(entity_id),
    )
}

fn missing_relation(
    relation_id: crate::identity::data::RelationId,
) -> RelationalMaterializationError {
    RelationalMaterializationError::SourceRecordUnavailable(
        crate::transactions::data::RecordRef::Relation(relation_id),
    )
}

fn suspension_batch(records: &[RelationalMaterializationRecord]) -> WorkerIntentBatch {
    records.iter().fold(
        WorkerIntentBatch::new("owner-materialization-suspension"),
        |batch, record| {
            batch.push(MutationIntent::Materialization(match record {
                RelationalMaterializationRecord::Entity { entity_id, .. } => {
                    MaterializationMutationIntent::SuspendEntity(
                        SuspendEntityMaterializationIntent {
                            entity_id: *entity_id,
                        },
                    )
                }
                RelationalMaterializationRecord::Relation { relation_id, .. } => {
                    MaterializationMutationIntent::SuspendRelation(
                        SuspendRelationMaterializationIntent {
                            relation_id: *relation_id,
                        },
                    )
                }
            }))
        },
    )
}

fn validate_custody_basis(
    basis: &AdmittedRelationalBranchBasis,
    custody: &RelationalMaterializationCustody,
) -> Result<(), RelationalMaterializationError> {
    if custody.data().branch_identity != *basis.identity() {
        return Err(RelationalMaterializationError::BranchMismatch);
    }
    if basis.inner.root.commit_id() != Some(custody.suspension_commit()) {
        return Err(RelationalMaterializationError::SuspensionCommitMismatch);
    }
    Ok(())
}

fn validate_candidate_manifest(
    custody: &RelationalMaterializationCustody,
    entities: &[RelationalEntityMaterialization],
    relations: &[RelationalRelationMaterialization],
) -> Result<(), RelationalMaterializationError> {
    let supplied = entities
        .iter()
        .map(|spec| RelationalMaterializationRecord::Entity {
            entity_id: spec.entity_id,
            kind_id: spec.kind_id,
        })
        .chain(
            relations
                .iter()
                .map(|spec| RelationalMaterializationRecord::Relation {
                    relation_id: spec.relation_id,
                    kind_id: spec.kind_id,
                    source: spec.source,
                    target: spec.target,
                }),
        )
        .collect::<BTreeSet<_>>();
    let expected = custody.records().iter().cloned().collect::<BTreeSet<_>>();
    if supplied.len() != entities.len() + relations.len() || supplied != expected {
        return Err(RelationalMaterializationError::CandidateManifestMismatch);
    }
    Ok(())
}

fn rematerialization_batch(
    entities: Vec<RelationalEntityMaterialization>,
    relations: Vec<RelationalRelationMaterialization>,
) -> WorkerIntentBatch {
    let batch = entities.into_iter().fold(
        WorkerIntentBatch::new("owner-materialization-restoration"),
        |batch, spec| {
            batch.push(MutationIntent::Materialization(
                MaterializationMutationIntent::RematerializeEntity(RematerializeEntityIntent {
                    entity_id: spec.entity_id,
                    kind_id: spec.kind_id,
                    fields: spec.fields,
                }),
            ))
        },
    );
    relations.into_iter().fold(batch, |batch, spec| {
        batch.push(MutationIntent::Materialization(
            MaterializationMutationIntent::RematerializeRelation(RematerializeRelationIntent {
                relation_id: spec.relation_id,
                kind_id: spec.kind_id,
                source: spec.source,
                target: spec.target,
                fields: spec.fields,
            }),
        ))
    })
}

#[cfg(test)]
#[path = "materialization_port_tests.rs"]
mod tests;
