//! Placing one staged intent into the overlay's indexes and the footprint.
//!
//! The overlay answers questions about records by looking them up, so every
//! staged intent must first be filed under the record it concerns, and the
//! transaction's footprint must learn the locus that intent claims. Those two
//! acts belong together: an intent filed under a record without a matching
//! locus would be invisible to admission, and a locus without a filing would
//! bound work the overlay cannot find. Whether an entity intent authors its
//! record or only observes it is not decided here: that answer belongs to
//! `intent_locus`, which staging admission reads as well, so the overlay and
//! the ceiling that bounds it cannot disagree about one staged intent.

use std::collections::BTreeMap;

use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, CreatedRelationRef, MutationIntent, RelationMutationIntent,
};

use super::intent_locus::{entity_intent_locus, EntityIntentLocus};
use super::overlay::IntentLocation;
use super::{
    RelationalTransactionFootprint, RelationalTransactionReadLocus, RelationalTransactionWriteLocus,
};

pub(super) fn index_intent(
    intent: &MutationIntent,
    location: IntentLocation,
    entity_mutations: &mut BTreeMap<EntityId, Vec<IntentLocation>>,
    relation_mutations: &mut BTreeMap<RelationId, Vec<IntentLocation>>,
    created_entities: &mut BTreeMap<CreatedEntityRef, Vec<IntentLocation>>,
    created_relations: &mut BTreeMap<CreatedRelationRef, Vec<IntentLocation>>,
    footprint: &mut RelationalTransactionFootprint,
) {
    match intent {
        MutationIntent::Entity(entity) => match entity_intent_locus(entity) {
            EntityIntentLocus::Write(id) => {
                entity_mutations.entry(id).or_default().push(location);
                footprint.record_write(RelationalTransactionWriteLocus::Existing(
                    crate::transactions::data::RecordRef::Entity(id),
                ));
            }
            EntityIntentLocus::Read(id) => {
                entity_mutations.entry(id).or_default().push(location);
                footprint.record_read(RelationalTransactionReadLocus::Existing(
                    crate::transactions::data::RecordRef::Entity(id),
                ));
            }
        },
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

fn relation_id(intent: &RelationMutationIntent) -> RelationId {
    match intent {
        RelationMutationIntent::UpdateEndpoints(intent) => intent.relation_id,
        RelationMutationIntent::ApplyAspectPatch(intent) => intent.relation_id,
        RelationMutationIntent::Delete(intent) => intent.relation_id,
    }
}
