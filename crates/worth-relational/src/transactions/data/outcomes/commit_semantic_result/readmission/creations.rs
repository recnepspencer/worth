//! Compare recorded canonical names with the original admitted create inputs.
use super::{Refusal, RelationalCommitSemanticResult};
use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::{
    CreateIntent, EntityReference, MutationIntent, RelationalCreatedEndpoint,
    RelationalCreatedEntityKey, RelationalCreatedRelationKey,
};

pub(super) fn validate_original_keys(
    result: &RelationalCommitSemanticResult,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Refusal> {
    let entity = |partition, kind, key| entity_key(envelope, partition, kind, key);
    let require_entity = |key| {
        if result
            .created_entities
            .iter()
            .any(|created| created.key == key)
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
            client_key: text(envelope, key)?,
            source: endpoint(envelope, source)?,
            target: endpoint(envelope, target)?,
        };
        if result
            .created_relations
            .iter()
            .any(|created| created.key == key)
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

fn text(envelope: &CanonicalCommitEnvelope, key: &ClientKey) -> Result<String, Refusal> {
    if let Some(value) = key.as_raw_str() {
        return Ok(value.to_owned());
    }
    key.as_symbol()
        .and_then(|symbol| {
            envelope
                .symbol_texts
                .entries
                .iter()
                .find(|entry| entry.0 == symbol)
        })
        .map(|(_, text)| text.as_str().to_owned())
        .ok_or(Refusal::MissingClientKeyText)
}

fn entity_key(
    envelope: &CanonicalCommitEnvelope,
    partition: PartitionId,
    kind: KindId,
    key: &ClientKey,
) -> Result<RelationalCreatedEntityKey, Refusal> {
    Ok(RelationalCreatedEntityKey {
        partition,
        kind,
        client_key: text(envelope, key)?,
    })
}

fn endpoint(
    envelope: &CanonicalCommitEnvelope,
    reference: &EntityReference,
) -> Result<RelationalCreatedEndpoint, Refusal> {
    match reference {
        EntityReference::Existing(id) => Ok(RelationalCreatedEndpoint::Existing(*id)),
        EntityReference::Created(created) => Ok(RelationalCreatedEndpoint::Created(entity_key(
            envelope,
            created.partition_id,
            created.kind_id,
            &created.client_key,
        )?)),
    }
}
