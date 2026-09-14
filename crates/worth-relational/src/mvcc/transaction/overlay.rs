use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{EntityId, RelationId};
use crate::symbols::data::{ClientKey, ClientKeySymbolPolicy, StringInterner, Symbol};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, WorkerIntentBatch,
};

use super::{RelationalTransactionFootprint, RelationalTransactionWriteLocus};

#[derive(Clone, Copy, Debug)]
struct IntentLocation {
    batch_index: usize,
    intent_index: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DetachedRelationalTransactionOverlay {
    batches: Vec<WorkerIntentBatch>,
    entity_mutations: BTreeMap<EntityId, Vec<IntentLocation>>,
    relation_mutations: BTreeMap<RelationId, Vec<IntentLocation>>,
    created_entities: BTreeMap<CreatedEntityRef, Vec<IntentLocation>>,
    created_relations: BTreeMap<CreatedRelationRef, Vec<IntentLocation>>,
    normalized_client_keys: BTreeMap<String, Symbol>,
}

impl DetachedRelationalTransactionOverlay {
    pub(crate) fn stage(
        &mut self,
        batch: WorkerIntentBatch,
        footprint: &mut RelationalTransactionFootprint,
    ) {
        self.batches.push(batch);
        self.index_batch(self.batches.len() - 1, footprint);
    }

    pub(crate) fn batches(&self) -> &[WorkerIntentBatch] {
        &self.batches
    }

    pub(crate) fn truncate_batches(
        &mut self,
        batch_len: usize,
        footprint: &mut RelationalTransactionFootprint,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Vec<WorkerIntentBatch> {
        let drained = self.batches.split_off(batch_len);
        *footprint = RelationalTransactionFootprint::for_basis(basis);
        self.rebuild_indexes(footprint);
        drained
    }

    pub(crate) fn entity_mutations(
        &self,
        entity: EntityId,
    ) -> impl Iterator<Item = &EntityMutationIntent> {
        self.entity_mutations
            .get(&entity)
            .into_iter()
            .flatten()
            .map(|location| match self.intent(*location) {
                MutationIntent::Entity(intent) => intent,
                _ => unreachable!("entity index points only to entity mutations"),
            })
    }

    pub(crate) fn relation_mutations(
        &self,
        relation: RelationId,
    ) -> impl Iterator<Item = &RelationMutationIntent> {
        self.relation_mutations
            .get(&relation)
            .into_iter()
            .flatten()
            .map(|location| match self.intent(*location) {
                MutationIntent::Relation(intent) => intent,
                _ => unreachable!("relation index points only to relation mutations"),
            })
    }

    pub(crate) fn canonical_created_entity_ref(
        &self,
        entity: &CreatedEntityRef,
    ) -> CreatedEntityRef {
        if self.created_entities.contains_key(entity) {
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
        if self.created_relations.contains_key(relation) {
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
        self.created_entities.get(entity).map(|locations| {
            locations
                .iter()
                .map(|location| match self.intent(*location) {
                    MutationIntent::Create(intent) => intent,
                    _ => unreachable!("created-entity index points only to creates"),
                })
        })
    }

    pub(crate) fn created_relation(
        &self,
        relation: &CreatedRelationRef,
    ) -> Option<impl ExactSizeIterator<Item = &CreateIntent>> {
        self.created_relations.get(relation).map(|locations| {
            locations
                .iter()
                .map(|location| match self.intent(*location) {
                    MutationIntent::Create(intent) => intent,
                    _ => unreachable!("created-relation index points only to creates"),
                })
        })
    }

    pub(crate) fn normalize_client_keys(
        &mut self,
        footprint: &mut RelationalTransactionFootprint,
        interner: &mut StringInterner,
        policy: ClientKeySymbolPolicy,
    ) -> Vec<(Symbol, String)> {
        if !policy.interns_requested_strings() {
            return Vec::new();
        }
        let mut raw_values = BTreeSet::new();
        for intent in self.batches.iter().flat_map(|batch| &batch.intents) {
            intent.collect_raw_client_keys(&mut raw_values);
        }
        footprint.collect_raw_client_keys(&mut raw_values);
        let mut new_snapshot_entries = Vec::new();
        for raw in raw_values {
            let was_present = interner.contains(&raw);
            let symbol = interner.intern(&raw);
            self.normalized_client_keys.insert(raw.clone(), symbol);
            if !was_present {
                new_snapshot_entries.push((symbol, raw));
            }
        }
        for intent in self.batches.iter_mut().flat_map(|batch| &mut batch.intents) {
            intent.normalize_client_keys(interner, policy);
        }
        footprint.normalize_created_loci(interner, policy);
        self.rebuild_indexes(footprint);
        new_snapshot_entries
    }

    fn rebuild_indexes(&mut self, footprint: &mut RelationalTransactionFootprint) {
        self.entity_mutations.clear();
        self.relation_mutations.clear();
        self.created_entities.clear();
        self.created_relations.clear();
        for batch_index in 0..self.batches.len() {
            self.index_batch(batch_index, footprint);
        }
    }

    fn index_batch(&mut self, batch_index: usize, footprint: &mut RelationalTransactionFootprint) {
        let batch = &self.batches[batch_index];
        for (intent_index, intent) in batch.intents.iter().enumerate() {
            let location = IntentLocation {
                batch_index,
                intent_index,
            };
            index_intent(
                intent,
                location,
                &mut self.entity_mutations,
                &mut self.relation_mutations,
                &mut self.created_entities,
                &mut self.created_relations,
                footprint,
            );
        }
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

fn index_intent(
    intent: &MutationIntent,
    location: IntentLocation,
    entity_mutations: &mut BTreeMap<EntityId, Vec<IntentLocation>>,
    relation_mutations: &mut BTreeMap<RelationId, Vec<IntentLocation>>,
    created_entities: &mut BTreeMap<CreatedEntityRef, Vec<IntentLocation>>,
    created_relations: &mut BTreeMap<CreatedRelationRef, Vec<IntentLocation>>,
    footprint: &mut RelationalTransactionFootprint,
) {
    match intent {
        MutationIntent::Entity(entity) => {
            let id = entity_id(entity);
            entity_mutations.entry(id).or_default().push(location);
            footprint.record_write(RelationalTransactionWriteLocus::Existing(
                crate::transactions::data::RecordRef::Entity(id),
            ));
        }
        MutationIntent::Relation(relation) => {
            let id = relation_id(relation);
            relation_mutations.entry(id).or_default().push(location);
            footprint.record_write(RelationalTransactionWriteLocus::Existing(
                crate::transactions::data::RecordRef::Relation(id),
            ));
        }
        MutationIntent::Create(create) => index_create(
            create,
            location,
            created_entities,
            created_relations,
            footprint,
        ),
        MutationIntent::Materialization(intent) => match intent.record() {
            crate::transactions::data::RecordRef::Entity(id) => {
                entity_mutations.entry(id).or_default().push(location);
                footprint.record_write(RelationalTransactionWriteLocus::Existing(
                    crate::transactions::data::RecordRef::Entity(id),
                ));
            }
            crate::transactions::data::RecordRef::Relation(id) => {
                relation_mutations.entry(id).or_default().push(location);
                footprint.record_write(RelationalTransactionWriteLocus::Existing(
                    crate::transactions::data::RecordRef::Relation(id),
                ));
            }
        },
    }
}

fn index_create(
    create: &CreateIntent,
    location: IntentLocation,
    created_entities: &mut BTreeMap<CreatedEntityRef, Vec<IntentLocation>>,
    created_relations: &mut BTreeMap<CreatedRelationRef, Vec<IntentLocation>>,
    footprint: &mut RelationalTransactionFootprint,
) {
    match create {
        CreateIntent::Entity(spec) => record_created_entity(
            CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            },
            location,
            created_entities,
            footprint,
        ),
        CreateIntent::EntityAspects(spec) => record_created_entity(
            CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            },
            location,
            created_entities,
            footprint,
        ),
        CreateIntent::BulkEntities(spec) => {
            for client_key in &spec.client_keys {
                record_created_entity(
                    CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                    },
                    location,
                    created_entities,
                    footprint,
                );
            }
        }
        CreateIntent::Relation(spec) => record_created_relation(
            CreatedRelationRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
                source: spec.source.clone(),
                target: spec.target.clone(),
            },
            location,
            created_relations,
            footprint,
        ),
        CreateIntent::RelationAspects(spec) => record_created_relation(
            CreatedRelationRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
                source: spec.source.clone(),
                target: spec.target.clone(),
            },
            location,
            created_relations,
            footprint,
        ),
        CreateIntent::BulkRelations(spec) => {
            for (client_key, (source, target)) in spec.client_keys.iter().zip(&spec.endpoints) {
                record_created_relation(
                    CreatedRelationRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                        source: source.clone(),
                        target: target.clone(),
                    },
                    location,
                    created_relations,
                    footprint,
                );
            }
        }
    }
}

fn record_created_entity(
    key: CreatedEntityRef,
    location: IntentLocation,
    created_entities: &mut BTreeMap<CreatedEntityRef, Vec<IntentLocation>>,
    footprint: &mut RelationalTransactionFootprint,
) {
    created_entities
        .entry(key.clone())
        .or_default()
        .push(location);
    footprint.record_write(RelationalTransactionWriteLocus::CreatedEntity(key));
}

fn record_created_relation(
    key: CreatedRelationRef,
    location: IntentLocation,
    created_relations: &mut BTreeMap<CreatedRelationRef, Vec<IntentLocation>>,
    footprint: &mut RelationalTransactionFootprint,
) {
    created_relations
        .entry(key.clone())
        .or_default()
        .push(location);
    footprint.record_write(RelationalTransactionWriteLocus::CreatedRelation(key));
}

fn entity_id(intent: &EntityMutationIntent) -> EntityId {
    match intent {
        EntityMutationIntent::UpdateFields(intent) => intent.entity_id,
        EntityMutationIntent::ApplyAspectPatch(intent) => intent.entity_id,
        EntityMutationIntent::Replace(intent) => intent.entity_id,
        EntityMutationIntent::Delete(intent) => intent.entity_id,
    }
}

fn relation_id(intent: &RelationMutationIntent) -> RelationId {
    match intent {
        RelationMutationIntent::UpdateEndpoints(intent) => intent.relation_id,
        RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
        RelationMutationIntent::Delete(intent) => intent.relation_id,
    }
}
