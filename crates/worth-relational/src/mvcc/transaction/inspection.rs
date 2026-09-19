//! Read-only inspection of transaction staging state.

use std::collections::BTreeSet;

use crate::inspection::data::{
    InspectionAccessPath, InspectionAvailability, InspectionOrigin, SavepointInspectionSurface,
    TransactionInspectionSurface, TransactionIntentCounts,
};
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent,
    TransactionId, WorkerIntentBatch,
};

pub(crate) fn inspect_staging_surface(
    transaction_id: TransactionId,
    target_branch: Option<crate::history::data::BranchId>,
    savepoints: &[crate::mvcc::RelationalTransactionSavepoint],
    batches: &[WorkerIntentBatch],
) -> TransactionInspectionSurface {
    let mut touched_records = BTreeSet::new();
    let mut intent_counts = TransactionIntentCounts::default();
    let mut reserved_bulk_entity_slots = 0_u64;
    let mut reserved_bulk_relation_slots = 0_u64;
    let mut contains_lineage_affecting_intents = false;

    for batch in batches {
        for intent in &batch.intents {
            match intent {
                MutationIntent::Create(create_intent) => {
                    intent_counts.create_count += 1;
                    match create_intent {
                        CreateIntent::Entity(_)
                        | CreateIntent::EntityAspects(_)
                        | CreateIntent::Relation(_)
                        | CreateIntent::RelationAspects(_) => {}
                        CreateIntent::BulkEntities(intent) => {
                            reserved_bulk_entity_slots += intent.field_patches.len() as u64;
                        }
                        CreateIntent::BulkRelations(intent) => {
                            reserved_bulk_relation_slots += intent.endpoints.len() as u64;
                        }
                    }
                }
                MutationIntent::Entity(entity_intent) => {
                    intent_counts.entity_mutation_count += 1;
                    contains_lineage_affecting_intents |=
                        matches!(entity_intent, EntityMutationIntent::Replace(_));
                    match entity_intent {
                        EntityMutationIntent::UpdateFields(intent) => {
                            touched_records.insert(RecordRef::Entity(intent.entity_id));
                        }
                        EntityMutationIntent::ApplyAspectPatch(intent) => {
                            touched_records.insert(RecordRef::Entity(intent.entity_id));
                        }
                        EntityMutationIntent::Replace(intent) => {
                            touched_records.insert(RecordRef::Entity(intent.entity_id));
                        }
                        EntityMutationIntent::Delete(intent) => {
                            touched_records.insert(RecordRef::Entity(intent.entity_id));
                        }
                    }
                }
                MutationIntent::Relation(relation_intent) => {
                    intent_counts.relation_mutation_count += 1;
                    match relation_intent {
                        RelationMutationIntent::UpdateEndpoints(intent) => {
                            touched_records.insert(RecordRef::Relation(intent.relation_id));
                        }
                        RelationMutationIntent::ApplyAspectPatch(intent) => {
                            touched_records.insert(RecordRef::Relation(intent.relation_id));
                        }
                        RelationMutationIntent::Delete(intent) => {
                            touched_records.insert(RecordRef::Relation(intent.relation_id));
                        }
                    }
                }
                MutationIntent::Materialization(intent) => match intent.record() {
                    RecordRef::Entity(entity_id) => {
                        intent_counts.entity_mutation_count += 1;
                        touched_records.insert(RecordRef::Entity(entity_id));
                    }
                    RecordRef::Relation(relation_id) => {
                        intent_counts.relation_mutation_count += 1;
                        touched_records.insert(RecordRef::Relation(relation_id));
                    }
                },
            }
        }
    }

    TransactionInspectionSurface {
        transaction_id,
        target_branch,
        batch_count: batches.len() as u64,
        savepoints: savepoints
            .iter()
            .map(|savepoint| SavepointInspectionSurface {
                savepoint_id: savepoint.id(),
                retained_batch_count: savepoint.retained_batch_count() as u64,
            })
            .collect(),
        touched_records: touched_records.into_iter().collect(),
        intent_counts,
        reserved_bulk_entity_slots,
        reserved_bulk_relation_slots,
        contains_lineage_affecting_intents,
        origin: InspectionOrigin::TransactionStaging,
        access_path: InspectionAccessPath::DirectLookup,
        availability: InspectionAvailability::Direct,
    }
}

impl crate::mvcc::BranchBoundRelationalTransaction {
    pub fn inspect_staging(&self) -> TransactionInspectionSurface {
        inspect_staging_surface(
            self.transaction_id,
            Some(self.basis.identity().branch_id().clone()),
            &self.savepoints,
            self.batches(),
        )
    }
}
