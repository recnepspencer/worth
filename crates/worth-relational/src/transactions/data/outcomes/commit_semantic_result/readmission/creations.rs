//! Compare recorded canonical names with the original admitted create inputs.
use super::{Refusal, RelationalCommitSemanticResult};
use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::StoredSymbolText;
use crate::symbols::data::{ClientKey, Symbol};
use crate::transactions::data::{
    CreateIntent, EntityReference, MutationIntent, RelationalCreatedEndpoint,
    RelationalCreatedEntityKey, RelationalCreatedRelationKey,
};
use std::cmp::Ordering;

pub(super) fn validate_original_keys(
    result: &RelationalCommitSemanticResult,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Refusal> {
    // Canonical ordering is established above. This ordinal roster preserves
    // the prior LAST value for duplicate symbols without cloning text or nodes.
    let symbols = SymbolNames::new(&envelope.symbol_texts.entries);
    let entity = |partition, kind, key: &ClientKey, (intent_ordinal, entity_ordinal)| {
        let text = symbols.text(key)?;
        result
            .created_entities
            .binary_search_by(|created| compare_entity_key(&created.key, partition, kind, text))
            .map(|_| ())
            .map_err(|_| Refusal::OriginalEntityKeyMissing {
                intent_ordinal,
                entity_ordinal,
            })
    };
    let require_relation = |partition,
                            kind,
                            key: &ClientKey,
                            source: &EntityReference,
                            target: &EntityReference,
                            (intent_ordinal, relation_ordinal)| {
        let text = symbols.text(key)?;
        let source_text = endpoint_text(&symbols, source)?;
        let target_text = endpoint_text(&symbols, target)?;
        result
            .created_relations
            .binary_search_by(|created| {
                compare_relation_key(
                    &created.key,
                    partition,
                    kind,
                    text,
                    source,
                    source_text,
                    target,
                    target_text,
                )
            })
            .map(|_| ())
            .map_err(|_| Refusal::OriginalRelationKeyMissing {
                intent_ordinal,
                relation_ordinal,
            })
    };
    for (intent_ordinal, intent) in envelope.merged_plan.merged_intents.iter().enumerate() {
        let MutationIntent::Create(create) = intent else {
            continue;
        };
        match create {
            CreateIntent::Entity(value) => {
                entity(
                    value.partition_id,
                    value.kind_id,
                    &value.client_key,
                    (intent_ordinal, 0),
                )?;
            }
            CreateIntent::EntityAspects(value) => {
                entity(
                    value.partition_id,
                    value.kind_id,
                    &value.client_key,
                    (intent_ordinal, 0),
                )?;
            }
            CreateIntent::BulkEntities(value) => {
                for (entity_ordinal, key) in value.client_keys.iter().enumerate() {
                    entity(
                        value.partition_id,
                        value.kind_id,
                        key,
                        (intent_ordinal, entity_ordinal),
                    )?;
                }
            }
            CreateIntent::Relation(value) => require_relation(
                value.partition_id,
                value.kind_id,
                &value.client_key,
                &value.source,
                &value.target,
                (intent_ordinal, 0),
            )?,
            CreateIntent::RelationAspects(value) => require_relation(
                value.partition_id,
                value.kind_id,
                &value.client_key,
                &value.source,
                &value.target,
                (intent_ordinal, 0),
            )?,
            CreateIntent::BulkRelations(value) => {
                if value.client_keys.len() != value.endpoints.len() {
                    return Err(Refusal::BulkRelationArityMismatch { intent_ordinal });
                }
                for (relation_ordinal, (key, (source, target))) in
                    value.client_keys.iter().zip(&value.endpoints).enumerate()
                {
                    require_relation(
                        value.partition_id,
                        value.kind_id,
                        key,
                        source,
                        target,
                        (intent_ordinal, relation_ordinal),
                    )?;
                }
            }
        }
    }
    Ok(())
}

struct SymbolNames<'a> {
    entries: &'a [(Symbol, StoredSymbolText)],
    ordered: Vec<usize>,
}
impl<'a> SymbolNames<'a> {
    fn new(entries: &'a [(Symbol, StoredSymbolText)]) -> Self {
        let mut ordered = Vec::with_capacity(entries.len());
        ordered.extend(0..entries.len());
        ordered.sort_unstable_by_key(|index| (entries[*index].0, *index));
        Self { entries, ordered }
    }
    fn text<'key>(&'key self, key: &'key ClientKey) -> Result<&'key str, Refusal> {
        if let Some(value) = key.as_raw_str() {
            return Ok(value);
        }
        let symbol = key.as_symbol().ok_or(Refusal::MissingClientKeyText)?;
        let end = self
            .ordered
            .partition_point(|index| self.entries[*index].0 <= symbol);
        let index = end
            .checked_sub(1)
            .and_then(|slot| self.ordered.get(slot))
            .filter(|index| self.entries[**index].0 == symbol)
            .ok_or(Refusal::MissingClientKeyText)?;
        Ok(self.entries[*index].1.as_str())
    }
}
fn compare_entity_key(
    recorded: &RelationalCreatedEntityKey,
    partition: PartitionId,
    kind: KindId,
    text: &str,
) -> Ordering {
    recorded
        .partition
        .cmp(&partition)
        .then(recorded.kind.cmp(&kind))
        .then(recorded.client_key.as_str().cmp(text))
}
fn endpoint_text<'a>(
    symbols: &'a SymbolNames<'_>,
    reference: &'a EntityReference,
) -> Result<Option<&'a str>, Refusal> {
    match reference {
        EntityReference::Existing(_) => Ok(None),
        EntityReference::Created(created) => symbols.text(&created.client_key).map(Some),
    }
}
fn compare_endpoint(
    recorded: &RelationalCreatedEndpoint,
    original: &EntityReference,
    text: Option<&str>,
) -> Ordering {
    match (recorded, original) {
        (RelationalCreatedEndpoint::Existing(left), EntityReference::Existing(right)) => {
            left.cmp(right)
        }
        (RelationalCreatedEndpoint::Existing(_), EntityReference::Created(_)) => Ordering::Less,
        (RelationalCreatedEndpoint::Created(_), EntityReference::Existing(_)) => Ordering::Greater,
        (RelationalCreatedEndpoint::Created(left), EntityReference::Created(right)) => {
            let text = text.expect("created endpoint text was resolved before binary comparison");
            compare_entity_key(left, right.partition_id, right.kind_id, text)
        }
    }
}
fn compare_relation_key(
    recorded: &RelationalCreatedRelationKey,
    partition: PartitionId,
    kind: KindId,
    text: &str,
    source: &EntityReference,
    source_text: Option<&str>,
    target: &EntityReference,
    target_text: Option<&str>,
) -> Ordering {
    recorded
        .partition
        .cmp(&partition)
        .then(recorded.kind.cmp(&kind))
        .then(recorded.client_key.as_str().cmp(text))
        .then_with(|| compare_endpoint(&recorded.source, source, source_text))
        .then_with(|| compare_endpoint(&recorded.target, target, target_text))
}
