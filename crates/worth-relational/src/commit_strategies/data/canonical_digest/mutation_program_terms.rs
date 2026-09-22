use super::{
    commit_strategy_digest, portable_aspect_patch_terms::write_portable_aspect_patch,
    StrategyDigestBytes,
};
use crate::commit_strategies::data::StrategyMutationProgramDigest;
use crate::transactions::data::{
    ApplyEntityAspectPatchIntent, ApplyRelationAspectPatchIntent, AspectFieldPatch,
    BulkEntityCreateIntent, BulkRelationCreateIntent, CreateIntent, DeleteEntityIntent,
    DeleteRelationIntent, EntityAspectCreateIntent, EntityMutationIntent, EntityReference,
    EntitySpec, MaterializationMutationIntent, MutationIntent, RelationAspectCreateIntent,
    RelationMutationIntent, RelationSpec, ReplaceEntityIntent, RevalidateEntityIntent,
    UpdateEntityFieldsIntent, UpdateRelationEndpointsIntent, WorkerIntentBatch,
};

pub(crate) fn strategy_mutation_program_digest(
    worker_batches: &[WorkerIntentBatch],
) -> StrategyMutationProgramDigest {
    StrategyMutationProgramDigest(commit_strategy_digest(
        "strategy-mutation-program-v1",
        |bytes| {
            bytes.usize(worker_batches.len());
            for batch in worker_batches {
                write_worker_batch(bytes, batch);
            }
        },
    ))
}

fn write_worker_batch(bytes: &mut StrategyDigestBytes, batch: &WorkerIntentBatch) {
    bytes.string(&batch.name);
    bytes.optional(batch.partition_key.as_deref(), |bytes, partition_key| {
        bytes.string(partition_key)
    });
    bytes.bool(batch.worker_local_only);
    bytes.usize(batch.intents.len());
    for intent in &batch.intents {
        write_mutation_intent(bytes, intent);
    }
}

fn write_mutation_intent(bytes: &mut StrategyDigestBytes, intent: &MutationIntent) {
    match intent {
        MutationIntent::Create(intent) => {
            bytes.tag(1);
            write_create_intent(bytes, intent);
        }
        MutationIntent::Entity(intent) => {
            bytes.tag(2);
            write_entity_mutation_intent(bytes, intent);
        }
        MutationIntent::Relation(intent) => {
            bytes.tag(3);
            write_relation_mutation_intent(bytes, intent);
        }
        MutationIntent::Materialization(intent) => {
            bytes.tag(4);
            write_materialization_intent(bytes, intent);
        }
    }
}

fn write_materialization_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &MaterializationMutationIntent,
) {
    match intent {
        MaterializationMutationIntent::SuspendEntity(intent) => {
            bytes.tag(1);
            bytes.entity_id(intent.entity_id);
        }
        MaterializationMutationIntent::SuspendRelation(intent) => {
            bytes.tag(2);
            bytes.relation_id(intent.relation_id);
        }
        MaterializationMutationIntent::RematerializeEntity(intent) => {
            bytes.tag(3);
            bytes.entity_id(intent.entity_id);
            bytes.kind_id(intent.kind_id);
            write_aspect_field_patch(bytes, &intent.fields);
        }
        MaterializationMutationIntent::RematerializeRelation(intent) => {
            bytes.tag(4);
            bytes.relation_id(intent.relation_id);
            bytes.kind_id(intent.kind_id);
            bytes.entity_id(intent.source);
            bytes.entity_id(intent.target);
            write_aspect_field_patch(bytes, &intent.fields);
        }
    }
}

fn write_create_intent(bytes: &mut StrategyDigestBytes, intent: &CreateIntent) {
    match intent {
        CreateIntent::Entity(intent) => {
            bytes.tag(1);
            write_entity_spec(bytes, intent);
        }
        CreateIntent::BulkEntities(intent) => {
            bytes.tag(2);
            write_bulk_entity_create_intent(bytes, intent);
        }
        CreateIntent::Relation(intent) => {
            bytes.tag(3);
            write_relation_spec(bytes, intent);
        }
        CreateIntent::BulkRelations(intent) => {
            bytes.tag(4);
            write_bulk_relation_create_intent(bytes, intent);
        }
        CreateIntent::EntityAspects(intent) => {
            bytes.tag(5);
            write_entity_aspect_create_intent(bytes, intent);
        }
        CreateIntent::RelationAspects(intent) => {
            bytes.tag(6);
            write_relation_aspect_create_intent(bytes, intent);
        }
    }
}

fn write_entity_mutation_intent(bytes: &mut StrategyDigestBytes, intent: &EntityMutationIntent) {
    match intent {
        EntityMutationIntent::UpdateFields(intent) => {
            bytes.tag(1);
            write_update_entity_fields_intent(bytes, intent);
        }
        EntityMutationIntent::Replace(intent) => {
            bytes.tag(2);
            write_replace_entity_intent(bytes, intent);
        }
        EntityMutationIntent::Delete(intent) => {
            bytes.tag(3);
            write_delete_entity_intent(bytes, intent);
        }
        EntityMutationIntent::ApplyAspectPatch(intent) => {
            bytes.tag(4);
            write_apply_entity_aspect_patch_intent(bytes, intent);
        }
        EntityMutationIntent::Revalidate(intent) => {
            bytes.tag(5);
            write_revalidate_entity_intent(bytes, intent);
        }
    }
}

fn write_relation_mutation_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &RelationMutationIntent,
) {
    match intent {
        RelationMutationIntent::UpdateEndpoints(intent) => {
            bytes.tag(1);
            write_update_relation_endpoints_intent(bytes, intent);
        }
        RelationMutationIntent::Delete(intent) => {
            bytes.tag(2);
            write_delete_relation_intent(bytes, intent);
        }
        RelationMutationIntent::ApplyAspectPatch(intent) => {
            bytes.tag(3);
            write_apply_relation_aspect_patch_intent(bytes, intent);
        }
    }
}

fn write_entity_spec(bytes: &mut StrategyDigestBytes, spec: &EntitySpec) {
    bytes.partition_id(spec.partition_id);
    bytes.kind_id(spec.kind_id);
    bytes.client_key(&spec.client_key);
    write_aspect_field_patch(bytes, &spec.fields);
}

fn write_relation_spec(bytes: &mut StrategyDigestBytes, spec: &RelationSpec) {
    bytes.partition_id(spec.partition_id);
    bytes.kind_id(spec.kind_id);
    bytes.client_key(&spec.client_key);
    write_entity_reference(bytes, &spec.source);
    write_entity_reference(bytes, &spec.target);
    write_aspect_field_patch(bytes, &spec.fields);
}

fn write_entity_aspect_create_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &EntityAspectCreateIntent,
) {
    bytes.partition_id(intent.partition_id);
    bytes.kind_id(intent.kind_id);
    bytes.client_key(&intent.client_key);
    write_portable_aspect_patch(bytes, &intent.aspect_patch);
}

fn write_relation_aspect_create_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &RelationAspectCreateIntent,
) {
    bytes.partition_id(intent.partition_id);
    bytes.kind_id(intent.kind_id);
    bytes.client_key(&intent.client_key);
    write_entity_reference(bytes, &intent.source);
    write_entity_reference(bytes, &intent.target);
    write_portable_aspect_patch(bytes, &intent.aspect_patch);
}

fn write_bulk_entity_create_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &BulkEntityCreateIntent,
) {
    bytes.partition_id(intent.partition_id);
    bytes.kind_id(intent.kind_id);
    write_client_keys(bytes, &intent.client_keys);
    write_aspect_field_patches(bytes, &intent.field_patches);
}

fn write_bulk_relation_create_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &BulkRelationCreateIntent,
) {
    bytes.partition_id(intent.partition_id);
    bytes.kind_id(intent.kind_id);
    write_client_keys(bytes, &intent.client_keys);
    bytes.usize(intent.endpoints.len());
    for (source, target) in &intent.endpoints {
        write_entity_reference(bytes, source);
        write_entity_reference(bytes, target);
    }
    write_aspect_field_patches(bytes, &intent.field_patches);
}

fn write_update_entity_fields_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &UpdateEntityFieldsIntent,
) {
    bytes.entity_id(intent.entity_id);
    write_aspect_field_patch(bytes, &intent.fields);
}

fn write_apply_entity_aspect_patch_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &ApplyEntityAspectPatchIntent,
) {
    bytes.entity_id(intent.entity_id);
    write_portable_aspect_patch(bytes, &intent.aspect_patch);
}

fn write_replace_entity_intent(bytes: &mut StrategyDigestBytes, intent: &ReplaceEntityIntent) {
    bytes.entity_id(intent.entity_id);
    write_entity_spec(bytes, &intent.replacement);
}

/// A revalidation demand names only the record it brings back under
/// judgement. Its tag is what separates it from the other intents that carry
/// nothing but an entity id, so two commits of opposite meaning over one
/// record cannot share a program digest.
fn write_revalidate_entity_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &RevalidateEntityIntent,
) {
    bytes.entity_id(intent.entity_id);
}

fn write_delete_entity_intent(bytes: &mut StrategyDigestBytes, intent: &DeleteEntityIntent) {
    bytes.entity_id(intent.entity_id);
}

fn write_update_relation_endpoints_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &UpdateRelationEndpointsIntent,
) {
    bytes.relation_id(intent.relation_id);
    bytes.kind_id(intent.kind_id);
    write_entity_reference(bytes, &intent.source);
    write_entity_reference(bytes, &intent.target);
}

fn write_apply_relation_aspect_patch_intent(
    bytes: &mut StrategyDigestBytes,
    intent: &ApplyRelationAspectPatchIntent,
) {
    bytes.relation_id(intent.relation_id);
    write_portable_aspect_patch(bytes, &intent.aspect_patch);
}

fn write_delete_relation_intent(bytes: &mut StrategyDigestBytes, intent: &DeleteRelationIntent) {
    bytes.relation_id(intent.relation_id);
}

fn write_entity_reference(bytes: &mut StrategyDigestBytes, reference: &EntityReference) {
    match reference {
        EntityReference::Existing(entity_id) => {
            bytes.tag(1);
            bytes.entity_id(*entity_id);
        }
        EntityReference::Created(created) => {
            bytes.tag(2);
            bytes.partition_id(created.partition_id);
            bytes.kind_id(created.kind_id);
            bytes.client_key(&created.client_key);
        }
    }
}

fn write_client_keys(
    bytes: &mut StrategyDigestBytes,
    client_keys: &[crate::symbols::data::ClientKey],
) {
    bytes.usize(client_keys.len());
    for client_key in client_keys {
        bytes.client_key(client_key);
    }
}

fn write_aspect_field_patches(bytes: &mut StrategyDigestBytes, patches: &[AspectFieldPatch]) {
    bytes.usize(patches.len());
    for patch in patches {
        write_aspect_field_patch(bytes, patch);
    }
}

fn write_aspect_field_patch(bytes: &mut StrategyDigestBytes, patch: &AspectFieldPatch) {
    match patch.to_canonical_bytes() {
        Ok(patch_bytes) => {
            bytes.tag(1);
            bytes.bytes(&patch_bytes);
        }
        Err(error) => {
            bytes.tag(2);
            bytes.string(error.detail());
        }
    }
}
