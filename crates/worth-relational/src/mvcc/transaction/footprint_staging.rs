use std::collections::BTreeSet;

use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityMutationIntent, EntityReference,
    MutationIntent, RecordRef, RelationMutationIntent, WorkerIntentBatch,
};

use super::{
    RelationalTransactionFootprint, RelationalTransactionReadLocus,
    RelationalTransactionStagingDenial, RelationalTransactionWriteLocus,
};

impl RelationalTransactionFootprint {
    pub(crate) fn admit_read(
        &mut self,
        locus: RelationalTransactionReadLocus,
        maximum_loci: usize,
    ) -> Result<(), RelationalTransactionStagingDenial> {
        let required_loci = self.total_locus_count() + usize::from(!self.reads.contains(&locus));
        if required_loci > maximum_loci {
            return Err(
                RelationalTransactionStagingDenial::FootprintCapacityExhausted {
                    maximum_loci,
                    required_loci,
                },
            );
        }
        self.reads.insert(locus);
        Ok(())
    }

    pub(crate) fn admit_staged_writes(
        &mut self,
        batch: &WorkerIntentBatch,
        maximum_loci: usize,
    ) -> Result<(), RelationalTransactionStagingDenial> {
        let staged = staged_write_loci(batch);
        let new_loci = staged
            .iter()
            .filter(|locus| !self.writes.contains(*locus))
            .count();
        let required_loci = self.total_locus_count().saturating_add(new_loci);
        if required_loci > maximum_loci {
            return Err(
                RelationalTransactionStagingDenial::FootprintCapacityExhausted {
                    maximum_loci,
                    required_loci,
                },
            );
        }
        for locus in staged {
            self.record_write(locus);
        }
        Ok(())
    }
}

fn staged_write_loci(batch: &WorkerIntentBatch) -> BTreeSet<RelationalTransactionWriteLocus> {
    let mut loci = BTreeSet::new();
    for intent in &batch.intents {
        match intent {
            MutationIntent::Entity(intent) => {
                let entity_id = match intent {
                    EntityMutationIntent::UpdateFields(intent) => intent.entity_id,
                    EntityMutationIntent::ApplyAspectPatch(intent) => intent.entity_id,
                    EntityMutationIntent::Replace(intent) => intent.entity_id,
                    EntityMutationIntent::Delete(intent) => intent.entity_id,
                };
                loci.insert(RelationalTransactionWriteLocus::Existing(
                    RecordRef::Entity(entity_id),
                ));
            }
            MutationIntent::Relation(intent) => {
                let relation_id = match intent {
                    RelationMutationIntent::UpdateEndpoints(intent) => intent.relation_id,
                    RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
                    RelationMutationIntent::Delete(intent) => intent.relation_id,
                };
                loci.insert(RelationalTransactionWriteLocus::Existing(
                    RecordRef::Relation(relation_id),
                ));
            }
            MutationIntent::Create(create) => collect_created_write_loci(create, &mut loci),
            MutationIntent::Materialization(intent) => {
                loci.insert(RelationalTransactionWriteLocus::Existing(intent.record()));
            }
        }
    }
    loci
}

fn collect_created_write_loci(
    create: &CreateIntent,
    loci: &mut BTreeSet<RelationalTransactionWriteLocus>,
) {
    match create {
        CreateIntent::Entity(spec) => {
            insert_created_entity(spec.partition_id, spec.kind_id, &spec.client_key, loci)
        }
        CreateIntent::EntityAspects(spec) => {
            insert_created_entity(spec.partition_id, spec.kind_id, &spec.client_key, loci)
        }
        CreateIntent::BulkEntities(spec) => {
            for key in &spec.client_keys {
                insert_created_entity(spec.partition_id, spec.kind_id, key, loci);
            }
        }
        CreateIntent::Relation(spec) => insert_created_relation(
            spec.partition_id,
            spec.kind_id,
            &spec.client_key,
            &spec.source,
            &spec.target,
            loci,
        ),
        CreateIntent::RelationAspects(spec) => insert_created_relation(
            spec.partition_id,
            spec.kind_id,
            &spec.client_key,
            &spec.source,
            &spec.target,
            loci,
        ),
        CreateIntent::BulkRelations(spec) => {
            for (key, (source, target)) in spec.client_keys.iter().zip(&spec.endpoints) {
                insert_created_relation(spec.partition_id, spec.kind_id, key, source, target, loci);
            }
        }
    }
}

fn insert_created_entity(
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: &ClientKey,
    loci: &mut BTreeSet<RelationalTransactionWriteLocus>,
) {
    loci.insert(RelationalTransactionWriteLocus::CreatedEntity(
        CreatedEntityRef {
            partition_id,
            kind_id,
            client_key: client_key.clone(),
        },
    ));
}

fn insert_created_relation(
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: &ClientKey,
    source: &EntityReference,
    target: &EntityReference,
    loci: &mut BTreeSet<RelationalTransactionWriteLocus>,
) {
    loci.insert(RelationalTransactionWriteLocus::CreatedRelation(
        CreatedRelationRef {
            partition_id,
            kind_id,
            client_key: client_key.clone(),
            source: source.clone(),
            target: target.clone(),
        },
    ));
}
