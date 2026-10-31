//! Compare recorded canonical names with the original admitted create inputs.
use super::{Refusal, RelationalCommitSemanticResult};
use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::{ClientKey, Symbol};
use crate::transactions::data::{
    CreateIntent, EntityReference, MutationIntent, RelationalCreatedEndpoint,
    RelationalCreatedEntityKey, RelationalCreatedRelationKey,
};
use std::collections::BTreeMap;

pub(super) fn validate_original_keys(
    result: &RelationalCommitSemanticResult,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Refusal> {
    // The enclosing validator has established canonical strict key ordering.
    // Build the borrowed dictionary lookup once; snapshot wire order is not
    // assumed to be symbol order.
    let symbols = envelope
        .symbol_texts
        .entries
        .iter()
        .map(|(symbol, text)| (*symbol, text.as_str()))
        .collect::<BTreeMap<_, _>>();
    let entity = |partition, kind, key| entity_key(&symbols, partition, kind, key);
    let require_entity = |key| {
        if result
            .created_entities
            .binary_search_by(|created| created.key.cmp(&key))
            .is_ok()
        {
            Ok(())
        } else {
            Err(Refusal::ConflictingCreateCorrespondence)
        }
    };
    let require_relation = |partition, kind, key: &ClientKey, source, target| {
        let key = RelationalCreatedRelationKey {
            partition,
            kind,
            client_key: text(&symbols, key)?,
            source: endpoint(&symbols, source)?,
            target: endpoint(&symbols, target)?,
        };
        if result
            .created_relations
            .binary_search_by(|created| created.key.cmp(&key))
            .is_ok()
        {
            Ok(())
        } else {
            Err(Refusal::ConflictingCreateCorrespondence)
        }
    };
    for intent in &envelope.merged_plan.merged_intents {
        let MutationIntent::Create(create) = intent else {
            continue;
        };
        match create {
            CreateIntent::Entity(value) => {
                require_entity(entity(
                    value.partition_id,
                    value.kind_id,
                    &value.client_key,
                )?)?;
            }
            CreateIntent::EntityAspects(value) => {
                require_entity(entity(
                    value.partition_id,
                    value.kind_id,
                    &value.client_key,
                )?)?;
            }
            CreateIntent::BulkEntities(value) => {
                for key in &value.client_keys {
                    require_entity(entity(value.partition_id, value.kind_id, key)?)?;
                }
            }
            CreateIntent::Relation(value) => require_relation(
                value.partition_id,
                value.kind_id,
                &value.client_key,
                &value.source,
                &value.target,
            )?,
            CreateIntent::RelationAspects(value) => require_relation(
                value.partition_id,
                value.kind_id,
                &value.client_key,
                &value.source,
                &value.target,
            )?,
            CreateIntent::BulkRelations(value) => {
                if value.client_keys.len() != value.endpoints.len() {
                    return Err(Refusal::ConflictingCreateCorrespondence);
                }
                for (key, (source, target)) in value.client_keys.iter().zip(&value.endpoints) {
                    require_relation(value.partition_id, value.kind_id, key, source, target)?;
                }
            }
        }
    }
    Ok(())
}

fn text(symbols: &BTreeMap<Symbol, &str>, key: &ClientKey) -> Result<String, Refusal> {
    if let Some(value) = key.as_raw_str() {
        return Ok(value.to_owned());
    }
    key.as_symbol()
        .and_then(|symbol| symbols.get(&symbol))
        .map(|text| (*text).to_owned())
        .ok_or(Refusal::MissingClientKeyText)
}

fn entity_key(
    symbols: &BTreeMap<Symbol, &str>,
    partition: PartitionId,
    kind: KindId,
    key: &ClientKey,
) -> Result<RelationalCreatedEntityKey, Refusal> {
    Ok(RelationalCreatedEntityKey {
        partition,
        kind,
        client_key: text(symbols, key)?,
    })
}

fn endpoint(
    symbols: &BTreeMap<Symbol, &str>,
    reference: &EntityReference,
) -> Result<RelationalCreatedEndpoint, Refusal> {
    match reference {
        EntityReference::Existing(id) => Ok(RelationalCreatedEndpoint::Existing(*id)),
        EntityReference::Created(created) => Ok(RelationalCreatedEndpoint::Created(entity_key(
            symbols,
            created.partition_id,
            created.kind_id,
            &created.client_key,
        )?)),
    }
}
