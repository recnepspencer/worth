use std::collections::BTreeSet;

use crate::symbols::data::{ClientKey, ClientKeySymbolPolicy, StringInterner};

use super::{CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent};
use crate::transactions::data::EntityReference;

impl MutationIntent {
    pub(crate) fn collect_raw_client_keys(&self, raw_values: &mut BTreeSet<String>) {
        match self {
            Self::Create(CreateIntent::Entity(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
            }
            Self::Create(CreateIntent::EntityAspects(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
            }
            Self::Create(CreateIntent::BulkEntities(spec)) => {
                collect_bulk_raw_client_keys(&spec.client_keys, raw_values);
            }
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                collect_bulk_raw_client_keys(&spec.client_keys, raw_values);
                for (source, target) in &spec.endpoints {
                    collect_entity_reference_raw_client_key(source, raw_values);
                    collect_entity_reference_raw_client_key(target, raw_values);
                }
            }
            Self::Create(CreateIntent::Relation(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
                collect_entity_reference_raw_client_key(&spec.source, raw_values);
                collect_entity_reference_raw_client_key(&spec.target, raw_values);
            }
            Self::Create(CreateIntent::RelationAspects(spec)) => {
                if let Some(raw) = spec.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
                collect_entity_reference_raw_client_key(&spec.source, raw_values);
                collect_entity_reference_raw_client_key(&spec.target, raw_values);
            }
            Self::Entity(EntityMutationIntent::Replace(spec)) => {
                if let Some(raw) = spec.replacement.client_key.as_raw_str() {
                    raw_values.insert(raw.to_string());
                }
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                collect_entity_reference_raw_client_key(&spec.source, raw_values);
                collect_entity_reference_raw_client_key(&spec.target, raw_values);
            }
            Self::Entity(_)
            | Self::Materialization(_)
            | Self::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | Self::Relation(RelationMutationIntent::Delete(_)) => {}
        }
    }

    pub(crate) fn normalize_client_keys(
        &mut self,
        interner: &mut StringInterner,
        policy: ClientKeySymbolPolicy,
    ) {
        match self {
            Self::Create(CreateIntent::Entity(spec)) => {
                spec.client_key = normalize_client_key(interner, policy, spec.client_key.clone());
            }
            Self::Create(CreateIntent::EntityAspects(spec)) => {
                spec.client_key = normalize_client_key(interner, policy, spec.client_key.clone());
            }
            Self::Create(CreateIntent::BulkEntities(spec)) => {
                normalize_bulk_client_keys(&mut spec.client_keys, interner, policy);
            }
            Self::Create(CreateIntent::BulkRelations(spec)) => {
                normalize_bulk_client_keys(&mut spec.client_keys, interner, policy);
                for (source, target) in &mut spec.endpoints {
                    normalize_entity_reference_client_key(source, interner, policy);
                    normalize_entity_reference_client_key(target, interner, policy);
                }
            }
            Self::Create(CreateIntent::Relation(spec)) => {
                spec.client_key = normalize_client_key(interner, policy, spec.client_key.clone());
                normalize_entity_reference_client_key(&mut spec.source, interner, policy);
                normalize_entity_reference_client_key(&mut spec.target, interner, policy);
            }
            Self::Create(CreateIntent::RelationAspects(spec)) => {
                spec.client_key = normalize_client_key(interner, policy, spec.client_key.clone());
                normalize_entity_reference_client_key(&mut spec.source, interner, policy);
                normalize_entity_reference_client_key(&mut spec.target, interner, policy);
            }
            Self::Entity(EntityMutationIntent::Replace(spec)) => {
                spec.replacement.client_key =
                    normalize_client_key(interner, policy, spec.replacement.client_key.clone());
            }
            Self::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                normalize_entity_reference_client_key(&mut spec.source, interner, policy);
                normalize_entity_reference_client_key(&mut spec.target, interner, policy);
            }
            Self::Entity(_)
            | Self::Materialization(_)
            | Self::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | Self::Relation(RelationMutationIntent::Delete(_)) => {}
        }
    }
}

fn collect_bulk_raw_client_keys(client_keys: &[ClientKey], raw_values: &mut BTreeSet<String>) {
    for client_key in client_keys {
        if let Some(raw) = client_key.as_raw_str() {
            raw_values.insert(raw.to_string());
        }
    }
}

fn normalize_bulk_client_keys(
    client_keys: &mut [ClientKey],
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) {
    for client_key in client_keys {
        *client_key = normalize_client_key(interner, policy, client_key.clone());
    }
}

fn collect_entity_reference_raw_client_key(
    entity_reference: &EntityReference,
    raw_values: &mut BTreeSet<String>,
) {
    if let EntityReference::Created(created) = entity_reference {
        if let Some(raw) = created.client_key.as_raw_str() {
            raw_values.insert(raw.to_string());
        }
    }
}

fn normalize_entity_reference_client_key(
    entity_reference: &mut EntityReference,
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) {
    if let EntityReference::Created(created) = entity_reference {
        created.client_key = normalize_client_key(interner, policy, created.client_key.clone());
    }
}

fn normalize_client_key(
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
    value: ClientKey,
) -> ClientKey {
    value.normalize_with(interner, policy)
}
