//! Native typed indexing; key construction retains existing ClientKey/endpoint
//! semantics. Newly cloned nested keys remain explicitly outside backing admission.
use super::{
    index_row::{IndexKey, IndexRow, IntentLocation},
    intent_locus::{entity_intent_locus, EntityIntentLocus},
    RelationalTransactionStagingDenial as Denial,
};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, MutationIntent, RecordRef,
    RelationMutationIntent, WorkerIntentBatch,
};

pub(super) fn index_batch(
    batch: &WorkerIntentBatch,
    batch_index: usize,
    mut emit: impl FnMut(IndexRow) -> Result<(), Denial>,
) -> Result<(), Denial> {
    let mut ordinal = 0usize;
    for (intent_index, intent) in batch.intents.iter().enumerate() {
        let location = IntentLocation {
            batch_index,
            intent_index,
        };
        index_intent(intent, &mut |key, observes| {
            let next = ordinal.checked_add(1).ok_or(Denial::CardinalityOverflow)?;
            emit(IndexRow {
                key,
                location,
                ordinal,
                observes,
            })?;
            ordinal = next;
            Ok(())
        })?;
    }
    Ok(())
}
fn index_intent(
    intent: &MutationIntent,
    emit: &mut impl FnMut(IndexKey, bool) -> Result<(), Denial>,
) -> Result<(), Denial> {
    match intent {
        MutationIntent::Entity(entity) => match entity_intent_locus(entity) {
            EntityIntentLocus::Write(id) => emit(IndexKey::Entity(id), false)?,
            EntityIntentLocus::Read(id) => emit(IndexKey::Entity(id), true)?,
        },
        MutationIntent::Relation(relation) => {
            let id = match relation {
                RelationMutationIntent::UpdateEndpoints(intent) => intent.relation_id,
                RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
                RelationMutationIntent::Delete(intent) => intent.relation_id,
            };
            emit(IndexKey::Relation(id), false)?;
        }
        MutationIntent::Materialization(intent) => match intent.record() {
            RecordRef::Entity(id) => emit(IndexKey::Entity(id), false)?,
            RecordRef::Relation(id) => emit(IndexKey::Relation(id), false)?,
        },
        MutationIntent::Create(create) => index_create(create, emit)?,
    }
    Ok(())
}
fn index_create(
    create: &CreateIntent,
    emit: &mut impl FnMut(IndexKey, bool) -> Result<(), Denial>,
) -> Result<(), Denial> {
    match create {
        CreateIntent::Entity(spec) => emit(
            IndexKey::CreatedEntity(CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            }),
            false,
        )?,
        CreateIntent::EntityAspects(spec) => emit(
            IndexKey::CreatedEntity(CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            }),
            false,
        )?,
        CreateIntent::BulkEntities(spec) => {
            for client_key in &spec.client_keys {
                emit(
                    IndexKey::CreatedEntity(CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                    }),
                    false,
                )?;
            }
        }
        CreateIntent::Relation(spec) => emit(
            IndexKey::CreatedRelation(CreatedRelationRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
                source: spec.source.clone(),
                target: spec.target.clone(),
            }),
            false,
        )?,
        CreateIntent::RelationAspects(spec) => emit(
            IndexKey::CreatedRelation(CreatedRelationRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
                source: spec.source.clone(),
                target: spec.target.clone(),
            }),
            false,
        )?,
        CreateIntent::BulkRelations(spec) => {
            for (client_key, (source, target)) in spec.client_keys.iter().zip(&spec.endpoints) {
                emit(
                    IndexKey::CreatedRelation(CreatedRelationRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                        source: source.clone(),
                        target: target.clone(),
                    }),
                    false,
                )?;
            }
        }
    }
    Ok(())
}
