use super::{
    index_row::{IndexKey, IndexRow, IntentLocation},
    staging_storage::{Author, OrderedStore},
    RelationalTransactionFootprint, RelationalTransactionStagingDenial as Denial,
};
use crate::identity::data::{EntityId, RelationId};
use crate::symbols::data::{ClientKey, Symbol};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, WorkerIntentBatch,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionAllocationPolicy;

#[derive(Clone, Debug, Default)]
pub(crate) struct DetachedRelationalTransactionOverlay {
    // Input batch/header storage remains System and outside the new backing claim.
    pub(super) batches: Vec<WorkerIntentBatch>,
    pub(super) index: OrderedStore<IndexRow>,
    pub(super) normalized_client_keys: BTreeMap<String, Symbol>,
    pub(super) normalization_generation: u64,
}
impl DetachedRelationalTransactionOverlay {
    pub(super) fn prepare_stage(
        &self,
        batch: &WorkerIntentBatch,
        footprint: &RelationalTransactionFootprint,
        maximum_loci: usize,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(OrderedStore<IndexRow>, RelationalTransactionFootprint), Denial> {
        let mut index = Author::new(&self.index, policy)?;
        let footprint =
            footprint.for_staged_batch(batch, self.batches.len(), maximum_loci, policy, |row| {
                index.insert(row)
            })?;
        Ok((index.finish()?, footprint))
    }
    pub(super) fn reserve_input_directory(&mut self) -> Result<(), Denial> {
        let requested_batches = self
            .batches
            .len()
            .checked_add(1)
            .ok_or(Denial::CardinalityOverflow)?;
        self.batches
            .try_reserve(1)
            .map_err(|_| Denial::InputDirectoryAllocationDenied { requested_batches })
    }
    pub(super) fn stage(&mut self, batch: WorkerIntentBatch, index: OrderedStore<IndexRow>) {
        self.batches.push(batch); // System capacity was reserved before publication
        self.index = index;
    }
    pub(crate) fn batches(&self) -> &[WorkerIntentBatch] {
        &self.batches
    }
    pub(super) fn prefix_index(&self, batch_len: usize) -> Result<OrderedStore<IndexRow>, Denial> {
        let mut index = Author::new(
            &OrderedStore::default(),
            ExecutionAllocationPolicy::SystemAllocation,
        )?;
        for (batch_index, batch) in self.batches[..batch_len].iter().enumerate() {
            super::overlay_indexing::index_batch(batch, batch_index, |row| index.insert(row))?;
        }
        index.finish()
    }
    pub(super) fn truncate_batches(
        &mut self,
        batch_len: usize,
        index: OrderedStore<IndexRow>,
    ) -> Vec<WorkerIntentBatch> {
        // Historical System drained/output backing is still an explicit open obligation.
        let drained = self.batches.split_off(batch_len);
        self.index = index;
        drained
    }
    pub(crate) fn entity_mutations(
        &self,
        entity: EntityId,
    ) -> impl Iterator<Item = &EntityMutationIntent> {
        self.index
            .range_by(|row| match &row.key {
                IndexKey::Entity(key) => key.cmp(&entity),
                _ => std::cmp::Ordering::Greater,
            })
            .map(|row| match self.intent(row.location) {
                MutationIntent::Entity(intent) => intent,
                _ => unreachable!("entity index points only to entity mutations"),
            })
    }
    pub(crate) fn relation_mutations(
        &self,
        relation: RelationId,
    ) -> impl Iterator<Item = &RelationMutationIntent> {
        self.index
            .range_by(|row| match &row.key {
                IndexKey::Entity(_) => std::cmp::Ordering::Less,
                IndexKey::Relation(key) => key.cmp(&relation),
                _ => std::cmp::Ordering::Greater,
            })
            .map(|row| match self.intent(row.location) {
                MutationIntent::Relation(intent) => intent,
                _ => unreachable!("relation index points only to relation mutations"),
            })
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
    pub(crate) fn canonical_created_entity_ref(
        &self,
        entity: &CreatedEntityRef,
    ) -> CreatedEntityRef {
        if self.entity_creates(entity).len() > 0 {
            return entity.clone();
        }
        CreatedEntityRef {
            partition_id: entity.partition_id,
            kind_id: entity.kind_id,
            client_key: self.canonical_client_key(&entity.client_key),
        }
    }
    pub(crate) fn canonical_created_relation_ref(
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
    pub(crate) fn created_entity(
        &self,
        entity: &CreatedEntityRef,
    ) -> Option<impl ExactSizeIterator<Item = &CreateIntent>> {
        let rows = self.entity_creates(entity);
        (rows.len() > 0).then(|| {
            rows.map(|row| match self.intent(row.location) {
                MutationIntent::Create(intent) => intent,
                _ => unreachable!("created entity index points only to creates"),
            })
        })
    }
    pub(crate) fn created_relation(
        &self,
        relation: &CreatedRelationRef,
    ) -> Option<impl ExactSizeIterator<Item = &CreateIntent>> {
        let rows = self.relation_creates(relation);
        (rows.len() > 0).then(|| {
            rows.map(|row| match self.intent(row.location) {
                MutationIntent::Create(intent) => intent,
                _ => unreachable!("created relation index points only to creates"),
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
