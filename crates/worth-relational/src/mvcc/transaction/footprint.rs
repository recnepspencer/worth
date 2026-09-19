use std::collections::BTreeSet;

use crate::identity::data::{KindId, PartitionId};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityMutationIntent, EntityReference,
    MaterializationMutationIntent, MutationIntent, RecordRef, RelationMutationIntent,
};

use super::footprint_client_keys::{
    collect_created_entity_raw_key, collect_created_relation_raw_keys, normalize_read_locus,
    normalize_write_locus,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationalTransactionReadLocus {
    Existing(RecordRef),
    CreatedEntity(CreatedEntityRef),
    CreatedRelation(CreatedRelationRef),
    ValidationPartition(PartitionId),
    EntitySchema(KindId),
    RelationSchema(KindId),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationalTransactionWriteLocus {
    Existing(RecordRef),
    CreatedEntity(CreatedEntityRef),
    CreatedRelation(CreatedRelationRef),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalTransactionFootprint {
    basis: crate::branch::RelationalBranchBasisDescriptor,
    pub(super) reads: BTreeSet<RelationalTransactionReadLocus>,
    pub(super) writes: BTreeSet<RelationalTransactionWriteLocus>,
    write_partitions: BTreeSet<PartitionId>,
}

impl RelationalTransactionFootprint {
    pub(crate) fn for_basis(basis: &crate::branch::AdmittedRelationalBranchBasis) -> Self {
        Self {
            basis: basis.descriptor().clone(),
            reads: BTreeSet::new(),
            writes: BTreeSet::new(),
            write_partitions: BTreeSet::new(),
        }
    }

    pub fn basis(&self) -> &crate::branch::RelationalBranchBasisDescriptor {
        &self.basis
    }

    pub fn branch(&self) -> &crate::history::data::BranchId {
        self.basis.branch_id()
    }

    pub fn reference(&self) -> &crate::branch::RelationalBranchReferenceObservation {
        self.basis.reference()
    }

    pub fn reads(&self) -> impl ExactSizeIterator<Item = &RelationalTransactionReadLocus> {
        self.reads.iter()
    }

    pub fn writes(&self) -> impl ExactSizeIterator<Item = &RelationalTransactionWriteLocus> {
        self.writes.iter()
    }

    pub fn write_partitions(&self) -> impl ExactSizeIterator<Item = &PartitionId> {
        self.write_partitions.iter()
    }

    pub(crate) fn record_read(&mut self, locus: RelationalTransactionReadLocus) {
        self.reads.insert(locus);
    }

    pub(super) fn total_locus_count(&self) -> usize {
        self.reads.len().saturating_add(self.writes.len())
    }

    pub(crate) fn derive_validation_dependencies(
        &mut self,
        plan: &crate::transactions::data::MergedCommitPlan,
        maximum_loci: usize,
    ) -> Result<(), super::RelationalTransactionStagingDenial> {
        let mut candidate = self.clone();
        for intent in &plan.merged_intents {
            candidate.record_validation_intent(intent);
        }
        for partition in candidate.write_partitions.clone() {
            candidate.record_read(RelationalTransactionReadLocus::ValidationPartition(
                partition,
            ));
        }
        let required_loci = candidate.total_locus_count();
        if required_loci > maximum_loci {
            return Err(
                super::RelationalTransactionStagingDenial::FootprintCapacityExhausted {
                    maximum_loci,
                    required_loci,
                },
            );
        }
        *self = candidate;
        Ok(())
    }

    pub(crate) fn validation_partitions(&self) -> BTreeSet<PartitionId> {
        let mut partitions = self.write_partitions.clone();
        for read in &self.reads {
            match read {
                RelationalTransactionReadLocus::Existing(RecordRef::Entity(entity)) => {
                    partitions.insert(entity.partition_id);
                }
                RelationalTransactionReadLocus::Existing(RecordRef::Relation(relation)) => {
                    partitions.insert(relation.partition_id);
                }
                RelationalTransactionReadLocus::CreatedEntity(entity) => {
                    partitions.insert(entity.partition_id);
                }
                RelationalTransactionReadLocus::CreatedRelation(relation) => {
                    partitions.insert(relation.partition_id);
                }
                RelationalTransactionReadLocus::ValidationPartition(partition) => {
                    partitions.insert(*partition);
                }
                RelationalTransactionReadLocus::EntitySchema(_)
                | RelationalTransactionReadLocus::RelationSchema(_) => {}
            }
        }
        partitions
    }

    fn record_validation_intent(&mut self, intent: &MutationIntent) {
        match intent {
            MutationIntent::Create(create) => self.record_create_dependencies(create),
            MutationIntent::Entity(entity) => {
                let (record, replacement_kind) = match entity {
                    EntityMutationIntent::UpdateFields(intent) => (intent.entity_id, None),
                    EntityMutationIntent::ApplyAspectPatch(intent) => (intent.entity_id, None),
                    EntityMutationIntent::Replace(intent) => {
                        (intent.entity_id, Some(intent.replacement.kind_id))
                    }
                    EntityMutationIntent::Delete(intent) => (intent.entity_id, None),
                    EntityMutationIntent::Revalidate(intent) => (intent.entity_id, None),
                };
                self.record_read(RelationalTransactionReadLocus::Existing(RecordRef::Entity(
                    record,
                )));
                if let Some(kind) = replacement_kind {
                    self.record_read(RelationalTransactionReadLocus::EntitySchema(kind));
                }
            }
            MutationIntent::Relation(relation) => {
                let record = match relation {
                    RelationMutationIntent::UpdateEndpoints(intent) => {
                        self.record_read(RelationalTransactionReadLocus::RelationSchema(
                            intent.kind_id,
                        ));
                        self.record_entity_reference(&intent.source);
                        self.record_entity_reference(&intent.target);
                        intent.relation_id
                    }
                    RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
                    RelationMutationIntent::Delete(intent) => intent.relation_id,
                };
                self.record_read(RelationalTransactionReadLocus::Existing(
                    RecordRef::Relation(record),
                ));
            }
            MutationIntent::Materialization(intent) => {
                self.record_read(RelationalTransactionReadLocus::Existing(intent.record()));
                match intent {
                    MaterializationMutationIntent::RematerializeEntity(spec) => {
                        self.record_read(RelationalTransactionReadLocus::EntitySchema(spec.kind_id))
                    }
                    MaterializationMutationIntent::RematerializeRelation(spec) => {
                        self.record_read(RelationalTransactionReadLocus::RelationSchema(
                            spec.kind_id,
                        ));
                        self.record_read(RelationalTransactionReadLocus::Existing(
                            RecordRef::Entity(spec.source),
                        ));
                        self.record_read(RelationalTransactionReadLocus::Existing(
                            RecordRef::Entity(spec.target),
                        ));
                    }
                    MaterializationMutationIntent::SuspendEntity(_)
                    | MaterializationMutationIntent::SuspendRelation(_) => {}
                }
            }
        }
    }

    fn record_create_dependencies(&mut self, create: &CreateIntent) {
        match create {
            CreateIntent::Entity(intent) => {
                self.record_read(RelationalTransactionReadLocus::EntitySchema(intent.kind_id));
            }
            CreateIntent::EntityAspects(intent) => {
                self.record_read(RelationalTransactionReadLocus::EntitySchema(intent.kind_id));
            }
            CreateIntent::BulkEntities(intent) => {
                self.record_read(RelationalTransactionReadLocus::EntitySchema(intent.kind_id));
            }
            CreateIntent::Relation(intent) => {
                self.record_relation_create(intent.kind_id, &intent.source, &intent.target);
            }
            CreateIntent::RelationAspects(intent) => {
                self.record_relation_create(intent.kind_id, &intent.source, &intent.target);
            }
            CreateIntent::BulkRelations(intent) => {
                self.record_read(RelationalTransactionReadLocus::RelationSchema(
                    intent.kind_id,
                ));
                for (source, target) in &intent.endpoints {
                    self.record_entity_reference(source);
                    self.record_entity_reference(target);
                }
            }
        }
    }

    fn record_relation_create(
        &mut self,
        kind: KindId,
        source: &EntityReference,
        target: &EntityReference,
    ) {
        self.record_read(RelationalTransactionReadLocus::RelationSchema(kind));
        self.record_entity_reference(source);
        self.record_entity_reference(target);
    }

    fn record_entity_reference(&mut self, reference: &EntityReference) {
        let locus = match reference {
            EntityReference::Existing(entity) => {
                RelationalTransactionReadLocus::Existing(RecordRef::Entity(*entity))
            }
            EntityReference::Created(entity) => {
                RelationalTransactionReadLocus::CreatedEntity(entity.clone())
            }
        };
        self.record_read(locus);
    }

    pub(crate) fn record_write(&mut self, locus: RelationalTransactionWriteLocus) {
        match &locus {
            RelationalTransactionWriteLocus::Existing(RecordRef::Entity(entity)) => {
                self.write_partitions.insert(entity.partition_id);
            }
            RelationalTransactionWriteLocus::Existing(RecordRef::Relation(relation)) => {
                self.write_partitions.insert(relation.partition_id);
            }
            RelationalTransactionWriteLocus::CreatedEntity(entity) => {
                self.write_partitions.insert(entity.partition_id);
            }
            RelationalTransactionWriteLocus::CreatedRelation(relation) => {
                self.write_partitions.insert(relation.partition_id);
            }
        }
        self.writes.insert(locus);
    }

    pub(crate) fn collect_raw_client_keys(&self, raw_values: &mut BTreeSet<String>) {
        for read in &self.reads {
            match read {
                RelationalTransactionReadLocus::CreatedEntity(created) => {
                    collect_created_entity_raw_key(created, raw_values);
                }
                RelationalTransactionReadLocus::CreatedRelation(created) => {
                    collect_created_relation_raw_keys(created, raw_values);
                }
                RelationalTransactionReadLocus::Existing(_)
                | RelationalTransactionReadLocus::ValidationPartition(_)
                | RelationalTransactionReadLocus::EntitySchema(_)
                | RelationalTransactionReadLocus::RelationSchema(_) => {}
            }
        }
        for write in &self.writes {
            match write {
                RelationalTransactionWriteLocus::CreatedEntity(created) => {
                    collect_created_entity_raw_key(created, raw_values);
                }
                RelationalTransactionWriteLocus::CreatedRelation(created) => {
                    collect_created_relation_raw_keys(created, raw_values);
                }
                RelationalTransactionWriteLocus::Existing(_) => {}
            }
        }
    }

    pub(crate) fn normalize_created_loci(
        &mut self,
        interner: &mut crate::symbols::data::StringInterner,
        policy: crate::symbols::data::ClientKeySymbolPolicy,
    ) {
        self.reads = std::mem::take(&mut self.reads)
            .into_iter()
            .map(|read| normalize_read_locus(read, interner, policy))
            .collect();
        self.writes = std::mem::take(&mut self.writes)
            .into_iter()
            .map(|write| normalize_write_locus(write, interner, policy))
            .collect();
    }
}
