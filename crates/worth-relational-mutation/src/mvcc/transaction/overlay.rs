use super::footprint::RelationalTransactionFootprint;
use super::index_row::IndexKey;
use super::index_row::IndexRow;
use super::index_row::IntentLocation;
use super::staged_mutations::RelationalTransactionStagedMutations as Staged;
use super::staging_storage::Author;
use super::staging_storage::OrderedStore;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial as Denial;
use crate::contracts::symbols::data::client_key::ClientKey;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use crate::contracts::transactions::data::primitives::{
    CreatedEntityRef, CreatedRelationRef, EntityReference, WorkerIntentBatch,
};
use worth_execution::ExecutionAllocationPolicy;
pub mod copy;

#[derive(Debug, Default)]
pub struct DetachedRelationalTransactionOverlay {
    pub effective_records: super::effective_records::EffectiveRecords,
    // Input batch/header storage remains System and outside the new backing claim.
    pub batches: Vec<WorkerIntentBatch>,
    pub index: OrderedStore<IndexRow>,
    pub normalized_client_keys: super::normalized_key_cache::NormalizedKeyCache,
    pub normalization_generation: u64,
    // Original independent input/cache data precedes its own funding owner.
    pub copied_input_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl DetachedRelationalTransactionOverlay {
    pub fn prepare_stage(
        &self,
        batch: &WorkerIntentBatch,
        footprint: &RelationalTransactionFootprint,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(OrderedStore<IndexRow>, RelationalTransactionFootprint), Denial> {
        let mut index = Author::new(&self.index, policy)?;
        let footprint = footprint
            .for_staged_batch(batch, self.batches.len(), policy, |row| index.insert(row))?;
        Ok((index.finish()?, footprint))
    }
    pub fn reserve_input_directory(&mut self) -> Result<(), Denial> {
        let requested_batches = self
            .batches
            .len()
            .checked_add(1)
            .ok_or(Denial::CardinalityOverflow)?;
        self.batches
            .try_reserve(1)
            .map_err(|_| Denial::InputDirectoryAllocationDenied { requested_batches })
    }
    pub fn stage(&mut self, batch: WorkerIntentBatch, index: OrderedStore<IndexRow>) {
        self.batches.push(batch); // System capacity was reserved before publication
        self.index = index;
    }
    pub fn staged_entity_mutations(&self, id: EntityId) -> Staged<EntityMutationIntent> {
        let prior = self.effective_records.entities.get(&id);
        self.extend_staged_history(IndexKey::Entity(id), prior, |intent| match intent {
            MutationIntent::Entity(intent) => Some(intent),
            _ => None,
        })
    }

    pub fn staged_relation_mutations(&self, id: RelationId) -> Staged<RelationMutationIntent> {
        let prior = self.effective_records.relations.get(&id);
        self.extend_staged_history(IndexKey::Relation(id), prior, |intent| match intent {
            MutationIntent::Relation(intent) => Some(intent),
            _ => None,
        })
    }

    fn extend_staged_history<I: Clone, V>(
        &self,
        key: IndexKey,
        prior: Option<&super::effective_records::RecordProjection<I, V>>,
        select: impl Fn(&MutationIntent) -> Option<&I>,
    ) -> Staged<I> {
        let (mut staged, through_batches) = prior
            .map(|p| (p.staged.clone(), p.through_batches))
            .unwrap_or_default();
        let suffix = self.index.range_by(|row| {
            row.key.cmp(&key).then_with(|| {
                if row.location.batch_index < through_batches {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
        });
        for row in suffix {
            if let Some(intent) = select(self.intent(row.location)) {
                staged.push(intent.clone());
            }
        }
        staged
    }

    pub fn batches(&self) -> &[WorkerIntentBatch] {
        &self.batches
    }
    pub fn prefix_index(&self, batch_len: usize) -> Result<OrderedStore<IndexRow>, Denial> {
        let mut index = Author::new(
            &OrderedStore::default(),
            ExecutionAllocationPolicy::SystemAllocation,
        )?;
        for (batch_index, batch) in self.batches[..batch_len].iter().enumerate() {
            super::overlay_indexing::index_batch(batch, batch_index, |row| index.insert(row))?;
        }
        index.finish()
    }
    pub fn truncate_batches(
        &mut self,
        batch_len: usize,
        index: OrderedStore<IndexRow>,
    ) -> Vec<WorkerIntentBatch> {
        // Historical System drained/output backing is still an explicit open obligation.
        let drained = self.batches.split_off(batch_len);
        self.index = index;
        drained
    }
    fn entity_creates(
        &self,
        entity: &CreatedEntityRef,
    ) -> super::staging_storage::OrderedIter<'_, IndexRow> {
        self.index.range_by(|row| match &row.key {
            IndexKey::Entity(_) | IndexKey::Relation(_) => std::cmp::Ordering::Less,
            IndexKey::CreatedEntity(key) => key.cmp(entity),
            _ => std::cmp::Ordering::Greater,
        })
    }
    fn relation_creates(
        &self,
        relation: &CreatedRelationRef,
    ) -> super::staging_storage::OrderedIter<'_, IndexRow> {
        self.index.range_by(|row| match &row.key {
            IndexKey::CreatedRelation(key) => key.cmp(relation),
            _ => std::cmp::Ordering::Less,
        })
    }
    pub fn canonical_created_entity_ref(&self, entity: &CreatedEntityRef) -> CreatedEntityRef {
        if self.entity_creates(entity).len() > 0 {
            return entity.clone();
        }
        CreatedEntityRef {
            partition_id: entity.partition_id,
            kind_id: entity.kind_id,
            client_key: self.canonical_client_key(&entity.client_key),
        }
    }
    pub fn canonical_created_relation_ref(
        &self,
        relation: &CreatedRelationRef,
    ) -> CreatedRelationRef {
        if self.relation_creates(relation).len() > 0 {
            return relation.clone();
        }
        CreatedRelationRef {
            partition_id: relation.partition_id,
            kind_id: relation.kind_id,
            client_key: self.canonical_client_key(&relation.client_key),
            source: self.canonical_entity_reference(&relation.source),
            target: self.canonical_entity_reference(&relation.target),
        }
    }
    pub fn created_entity(
        &self,
        entity: &CreatedEntityRef,
    ) -> Option<impl ExactSizeIterator<Item = &CreateIntent> + use<'_>> {
        let rows = self.entity_creates(entity);
        (rows.len() > 0).then(|| {
            rows.map(|row| match self.intent(row.location) {
                MutationIntent::Create(intent) => intent,
                _ => unreachable!(/* Proof: The private mutation index is populated only in the corresponding entity/relation/create insertion branch; its immutable index points to that discriminant. */ "created entity index points only to creates"),
            })
        })
    }
    pub fn created_relation(
        &self,
        relation: &CreatedRelationRef,
    ) -> Option<impl ExactSizeIterator<Item = &CreateIntent> + use<'_>> {
        let rows = self.relation_creates(relation);
        (rows.len() > 0).then(|| {
            rows.map(|row| match self.intent(row.location) {
                MutationIntent::Create(intent) => intent,
                _ => unreachable!(/* Proof: The private mutation index is populated only in the corresponding entity/relation/create insertion branch; its immutable index points to that discriminant. */ "created relation index points only to creates"),
            })
        })
    }
    fn intent(&self, location: IntentLocation) -> &MutationIntent {
        &self.batches[location.batch_index].intents[location.intent_index]
    }
    fn canonical_client_key(&self, key: &ClientKey) -> ClientKey {
        key.as_raw_str()
            .and_then(|raw| self.normalized_client_keys.get(raw).copied())
            .map(ClientKey::symbol)
            .unwrap_or_else(|| key.clone())
    }
    fn canonical_entity_reference(&self, reference: &EntityReference) -> EntityReference {
        match reference {
            EntityReference::Existing(entity) => EntityReference::Existing(*entity),
            EntityReference::Created(created) => {
                EntityReference::Created(self.canonical_created_entity_ref(created))
            }
        }
    }
}
