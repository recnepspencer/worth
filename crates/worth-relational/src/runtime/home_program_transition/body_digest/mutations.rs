//! Exhaustive authored mutation structure; never normalizes or executes it.
use super::{patches, BodyOutput, RelationalHomeProgramBodyDenial as Denial};
use crate::aspect_wire::CanonicalAspectOutput;
use crate::transactions::data::*;

pub(super) fn write(
    out: &mut BodyOutput<'_, '_, '_>,
    intent: &MutationIntent,
) -> Result<(), Denial> {
    match intent {
        MutationIntent::Create(intent) => {
            out.byte(0)?;
            create(out, intent)?;
        }
        MutationIntent::Entity(intent) => {
            out.byte(1)?;
            entity(out, intent)?;
        }
        MutationIntent::Relation(intent) => {
            out.byte(2)?;
            relation(out, intent)?;
        }
        MutationIntent::Materialization(_) => return Err(Denial::UnsupportedMaterialization),
    }
    Ok(())
}

fn create(out: &mut BodyOutput<'_, '_, '_>, intent: &CreateIntent) -> Result<(), Denial> {
    match intent {
        CreateIntent::Entity(spec) => {
            out.byte(0)?;
            entity_spec(out, spec)?;
        }
        CreateIntent::EntityAspects(EntityAspectCreateIntent {
            partition_id,
            kind_id,
            client_key,
            aspect_patch,
        }) => {
            out.byte(1)?;
            out.scalar(&(partition_id, kind_id, client_key))?;
            patches::portable(out, aspect_patch)?;
        }
        CreateIntent::BulkEntities(BulkEntityCreateIntent {
            partition_id,
            kind_id,
            client_keys,
            field_patches,
        }) => {
            out.byte(2)?;
            out.scalar(&(partition_id, kind_id))?;
            out.length(client_keys.len())?;
            for key in client_keys {
                out.scalar(key)?;
            }
            out.length(field_patches.len())?;
            for patch in field_patches {
                patches::fields(out, patch)?;
            }
        }
        CreateIntent::Relation(RelationSpec {
            partition_id,
            kind_id,
            client_key,
            source,
            target,
            fields,
        }) => {
            out.byte(3)?;
            out.scalar(&(partition_id, kind_id, client_key, source, target))?;
            patches::fields(out, fields)?;
        }
        CreateIntent::RelationAspects(RelationAspectCreateIntent {
            partition_id,
            kind_id,
            client_key,
            source,
            target,
            aspect_patch,
        }) => {
            out.byte(4)?;
            out.scalar(&(partition_id, kind_id, client_key, source, target))?;
            patches::portable(out, aspect_patch)?;
        }
        CreateIntent::BulkRelations(BulkRelationCreateIntent {
            partition_id,
            kind_id,
            client_keys,
            endpoints,
            field_patches,
        }) => {
            out.byte(5)?;
            out.scalar(&(partition_id, kind_id))?;
            out.length(client_keys.len())?;
            for key in client_keys {
                out.scalar(key)?;
            }
            out.length(endpoints.len())?;
            for pair in endpoints {
                out.scalar(pair)?;
            }
            out.length(field_patches.len())?;
            for patch in field_patches {
                patches::fields(out, patch)?;
            }
        }
    }
    Ok(())
}

fn entity_spec(out: &mut BodyOutput<'_, '_, '_>, spec: &EntitySpec) -> Result<(), Denial> {
    let EntitySpec {
        partition_id,
        kind_id,
        client_key,
        fields,
    } = spec;
    out.scalar(&(partition_id, kind_id, client_key))?;
    patches::fields(out, fields)
}

fn entity(out: &mut BodyOutput<'_, '_, '_>, intent: &EntityMutationIntent) -> Result<(), Denial> {
    match intent {
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }) => {
            out.byte(0)?;
            out.scalar(entity_id)?;
            patches::fields(out, fields)?;
        }
        EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
            entity_id,
            aspect_patch,
        }) => {
            out.byte(1)?;
            out.scalar(entity_id)?;
            patches::portable(out, aspect_patch)?;
        }
        EntityMutationIntent::Replace(ReplaceEntityIntent {
            entity_id,
            replacement,
        }) => {
            out.byte(2)?;
            out.scalar(entity_id)?;
            entity_spec(out, replacement)?;
        }
        EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }) => {
            out.byte(3)?;
            out.scalar(entity_id)?;
        }
        EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id }) => {
            out.byte(4)?;
            out.scalar(entity_id)?;
        }
    }
    Ok(())
}

fn relation(
    out: &mut BodyOutput<'_, '_, '_>,
    intent: &RelationMutationIntent,
) -> Result<(), Denial> {
    match intent {
        RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
            relation_id,
            kind_id,
            source,
            target,
        }) => {
            out.byte(0)?;
            out.scalar(&(relation_id, kind_id, source, target))?;
        }
        RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
            relation_id,
            aspect_patch,
        }) => {
            out.byte(1)?;
            out.scalar(relation_id)?;
            patches::portable(out, aspect_patch)?;
        }
        RelationMutationIntent::Delete(DeleteRelationIntent { relation_id }) => {
            out.byte(2)?;
            out.scalar(relation_id)?;
        }
        RelationMutationIntent::Revalidate(RevalidateRelationIntent { relation_id }) => {
            out.byte(3)?;
            out.scalar(relation_id)?;
        }
    }
    Ok(())
}
