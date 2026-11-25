use crate::commit_strategies::data::artifact_basis::StrategyArtifactBasis;
use std::collections::BTreeSet;

use worth_foundational::facade::AspectFieldLocator;

use super::canonical_digest::{
    native_entity_fields_scope_digest, native_entity_replacement_scope_digest,
};
use crate::commit_strategies::data::descriptor::descriptor_record::CommitStrategyDescriptor;
use crate::commit_strategies::data::native_codec::{
    decode_aspect_field_locator, decode_aspect_field_patch, decode_aspect_value, decode_entity_id,
    decode_string, NativeCodecReader,
};
use crate::identity::data::EntityId;
use crate::transactions::data::intents::mutation_intent::{EntityMutationIntent, MutationIntent};

#[derive(Debug)]
enum NativeStrategyIntentScope {
    EntityFields {
        entity_id: EntityId,
        targets: Vec<AspectFieldLocator>,
    },
    EntityReplacement {
        entity_id: EntityId,
        replacement_client_key: String,
        targets: Vec<AspectFieldLocator>,
    },
}

pub(super) fn native_strategy_intent_scope_digest(
    descriptor: &CommitStrategyDescriptor,
    lowered: &StrategyArtifactBasis,
) -> Option<[u8; 32]> {
    let scope = native_strategy_intent_scope(descriptor, lowered)?;
    let digest = match scope {
        NativeStrategyIntentScope::EntityFields { entity_id, targets } => {
            native_entity_fields_scope_digest(entity_id, &targets)
        }
        NativeStrategyIntentScope::EntityReplacement {
            entity_id,
            replacement_client_key,
            targets,
        } => native_entity_replacement_scope_digest(entity_id, &replacement_client_key, &targets),
    };
    Some(digest)
}

pub(super) fn native_strategy_intent_scope_targets(
    descriptor: &CommitStrategyDescriptor,
    lowered: &StrategyArtifactBasis,
) -> Option<Vec<AspectFieldLocator>> {
    let scope = native_strategy_intent_scope(descriptor, lowered)?;
    let mut targets = match scope {
        NativeStrategyIntentScope::EntityFields { targets, .. }
        | NativeStrategyIntentScope::EntityReplacement { targets, .. } => targets,
    };
    targets.sort();
    targets.dedup();
    Some(targets)
}

fn native_strategy_intent_scope(
    descriptor: &CommitStrategyDescriptor,
    lowered: &StrategyArtifactBasis,
) -> Option<NativeStrategyIntentScope> {
    let bytes = lowered.request().canonical_input().canonical_bytes();
    match descriptor.family_name().as_str() {
        "strategy.aspect" => native_aspect_field_scope(bytes, lowered),
        "strategy.replica" => native_replica_scope(bytes, lowered),
        "strategy.intent" => native_intent_reconciliation_scope(bytes, lowered),
        "strategy.replace" => native_entity_replacement_scope(bytes, lowered),
        _ => None,
    }
}

fn native_aspect_field_scope(
    bytes: &[u8],
    lowered: &StrategyArtifactBasis,
) -> Option<NativeStrategyIntentScope> {
    let mut reader = NativeCodecReader::new(bytes);
    let entity_id = decode_entity_id(&mut reader).ok()?;
    decode_aspect_field_locator(&mut reader).ok()?;
    decode_aspect_value(&mut reader).ok()?;
    reader.finish().ok()?;
    Some(NativeStrategyIntentScope::EntityFields {
        entity_id,
        targets: lowered_entity_patch_targets(lowered, entity_id)?,
    })
}

fn native_replica_scope(
    bytes: &[u8],
    lowered: &StrategyArtifactBasis,
) -> Option<NativeStrategyIntentScope> {
    let mut reader = NativeCodecReader::new(bytes);
    let entity_id = decode_entity_id(&mut reader).ok()?;
    reader.read_u64().ok()?;
    reader.finish().ok()?;
    Some(NativeStrategyIntentScope::EntityFields {
        entity_id,
        targets: lowered_entity_patch_targets(lowered, entity_id)?,
    })
}

fn native_intent_reconciliation_scope(
    bytes: &[u8],
    lowered: &StrategyArtifactBasis,
) -> Option<NativeStrategyIntentScope> {
    let mut reader = NativeCodecReader::new(bytes);
    let entity_id = decode_entity_id(&mut reader).ok()?;
    let desired_aspect_fields = decode_aspect_field_patch(&mut reader).ok()?;
    reader.finish().ok()?;
    Some(NativeStrategyIntentScope::EntityFields {
        entity_id,
        targets: lowered_entity_patch_targets(lowered, entity_id)
            .unwrap_or_else(|| desired_aspect_fields.locators().cloned().collect()),
    })
}

fn native_entity_replacement_scope(
    bytes: &[u8],
    lowered: &StrategyArtifactBasis,
) -> Option<NativeStrategyIntentScope> {
    let mut reader = NativeCodecReader::new(bytes);
    let entity_id = decode_entity_id(&mut reader).ok()?;
    let replacement_client_key = decode_string(&mut reader).ok()?;
    let desired_aspect_fields = decode_aspect_field_patch(&mut reader).ok()?;
    reader.finish().ok()?;
    Some(NativeStrategyIntentScope::EntityReplacement {
        entity_id,
        replacement_client_key,
        targets: lowered_entity_patch_targets(lowered, entity_id)
            .unwrap_or_else(|| desired_aspect_fields.locators().cloned().collect()),
    })
}

fn lowered_entity_patch_targets(
    lowered: &StrategyArtifactBasis,
    entity_id: EntityId,
) -> Option<Vec<AspectFieldLocator>> {
    let mut targets = BTreeSet::new();
    for intent in &lowered.merged_plan().merged_intents {
        match intent {
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
                if update.entity_id == entity_id =>
            {
                targets.extend(update.fields.locators().cloned());
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(replacement))
                if replacement.entity_id == entity_id =>
            {
                targets.extend(replacement.replacement.fields.locators().cloned());
            }
            _ => {}
        }
    }
    (!targets.is_empty()).then(|| targets.into_iter().collect())
}
