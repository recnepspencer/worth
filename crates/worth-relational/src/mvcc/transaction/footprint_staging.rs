use std::collections::BTreeSet;

use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityReference, MutationIntent, RecordRef,
    RelationMutationIntent, WorkerIntentBatch,
};

use super::intent_locus::{entity_intent_locus, EntityIntentLocus};
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

    /// Admits everything one staged batch will touch against the transaction's
    /// footprint capacity.
    ///
    /// Not every staged intent writes: a revalidation demand only brings a
    /// record back under judgement. Such a demand still costs the transaction a
    /// locus, and counting only writes would leave it the one staged intent no
    /// capacity bounds, so reads and writes are admitted together against the
    /// same ceiling.
    pub(crate) fn admit_staged_loci(
        &mut self,
        batch: &WorkerIntentBatch,
        maximum_loci: usize,
    ) -> Result<(), RelationalTransactionStagingDenial> {
        let staged = staged_loci(batch);
        let new_loci = staged
            .writes
            .iter()
            .filter(|locus| !self.writes.contains(*locus))
            .count()
            .saturating_add(
                staged
                    .reads
                    .iter()
                    .filter(|locus| !self.reads.contains(*locus))
                    .count(),
            );
        let required_loci = self.total_locus_count().saturating_add(new_loci);
        if required_loci > maximum_loci {
            return Err(
                RelationalTransactionStagingDenial::FootprintCapacityExhausted {
                    maximum_loci,
                    required_loci,
                },
            );
        }
        for locus in staged.writes {
            self.record_write(locus);
        }
        for locus in staged.reads {
            self.record_read(locus);
        }
        Ok(())
    }
}

/// What one staged batch will touch, separated by the authority each locus
/// carries: a write changes the record, a read only observes it.
#[derive(Default)]
struct StagedIntentLoci {
    writes: BTreeSet<RelationalTransactionWriteLocus>,
    reads: BTreeSet<RelationalTransactionReadLocus>,
}

fn staged_loci(batch: &WorkerIntentBatch) -> StagedIntentLoci {
    let mut loci = StagedIntentLoci::default();
    for intent in &batch.intents {
        match intent {
            MutationIntent::Entity(intent) => match entity_intent_locus(intent) {
                EntityIntentLocus::Write(entity_id) => {
                    loci.writes
                        .insert(RelationalTransactionWriteLocus::Existing(
                            RecordRef::Entity(entity_id),
                        ));
                }
                EntityIntentLocus::Read(entity_id) => {
                    loci.reads
                        .insert(RelationalTransactionReadLocus::Existing(RecordRef::Entity(
                            entity_id,
                        )));
                }
            },
            MutationIntent::Relation(intent) => {
                let relation_id = match intent {
                    RelationMutationIntent::UpdateEndpoints(intent) => intent.relation_id,
                    RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
                    RelationMutationIntent::Delete(intent) => intent.relation_id,
                };
                loci.writes
                    .insert(RelationalTransactionWriteLocus::Existing(
                        RecordRef::Relation(relation_id),
                    ));
            }
            MutationIntent::Create(create) => collect_created_write_loci(create, &mut loci.writes),
            MutationIntent::Materialization(intent) => {
                loci.writes
                    .insert(RelationalTransactionWriteLocus::Existing(intent.record()));
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
