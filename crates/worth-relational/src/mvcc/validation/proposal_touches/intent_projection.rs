use super::{
    ValidatedMutationTouch, ValidatedMutationTouchProjectionError,
    ValidatedMutationTouchProjectionWork,
};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{
    AspectFieldPatch, CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use worth_foundational::facade::PortableAspectPatchOperation;
pub(super) fn project_intent(
    intent: &MutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut super::collection::TouchCollection<'_, '_>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    match intent {
        MutationIntent::Create(intent) => {
            project_create(intent, touches);
            Ok(())
        }
        MutationIntent::Entity(intent) => project_entity(intent, state, touches, work),
        MutationIntent::Relation(intent) => project_relation(intent, state, touches, work),
        MutationIntent::Materialization(intent) => {
            project_materialization(intent, state, touches, work)
        }
    }
}

fn project_materialization(
    intent: &crate::transactions::data::MaterializationMutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut super::collection::TouchCollection<'_, '_>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    use crate::transactions::data::MaterializationMutationIntent as Intent;
    match intent {
        Intent::SuspendEntity(spec) => {
            let kind = entity_kind(state, spec.entity_id, work)?;
            touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
        }
        Intent::SuspendRelation(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::UnlinkRelation { kind });
        }
        Intent::RematerializeEntity(spec) => {
            touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation {
                kind: spec.kind_id,
            });
            project_fields(spec.kind_id, &spec.fields, touches);
        }
        Intent::RematerializeRelation(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            project_relation_fields(spec.kind_id, &spec.fields, touches);
        }
    }
    Ok(())
}

fn project_create(intent: &CreateIntent, touches: &mut super::collection::TouchCollection<'_, '_>) {
    match intent {
        CreateIntent::Entity(spec) => {
            touches.insert(ValidatedMutationTouch::CreateEntity { kind: spec.kind_id });
            project_fields(spec.kind_id, &spec.fields, touches);
        }
        CreateIntent::EntityAspects(spec) => {
            touches.insert(ValidatedMutationTouch::CreateEntity { kind: spec.kind_id });
            project_entity_aspect_patch(spec.kind_id, &spec.aspect_patch, touches);
        }
        CreateIntent::BulkEntities(spec) => {
            touches.insert(ValidatedMutationTouch::CreateEntity { kind: spec.kind_id });
            for fields in &spec.field_patches {
                if touches.failure.is_some() {
                    break;
                }
                project_fields(spec.kind_id, fields, touches);
            }
        }
        CreateIntent::Relation(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            project_relation_fields(spec.kind_id, &spec.fields, touches);
        }
        CreateIntent::RelationAspects(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            if !spec.aspect_patch.operations().is_empty() {
                touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation {
                    kind: spec.kind_id,
                });
            }
        }
        CreateIntent::BulkRelations(spec) => {
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
            for fields in &spec.field_patches {
                if touches.failure.is_some() {
                    break;
                }
                if let Err(error) = (touches.checkpoint)(1) {
                    touches.failure = Some(error);
                    break;
                }
                if fields.locators().next().is_some() {
                    touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation {
                        kind: spec.kind_id,
                    });
                    break;
                }
            }
        }
    }
}

fn project_entity(
    intent: &EntityMutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut super::collection::TouchCollection<'_, '_>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    let entity = match intent {
        EntityMutationIntent::UpdateFields(spec) => spec.entity_id,
        EntityMutationIntent::ApplyAspectPatch(spec) => spec.entity_id,
        EntityMutationIntent::Replace(spec) => spec.entity_id,
        EntityMutationIntent::Delete(spec) => spec.entity_id,
        EntityMutationIntent::Revalidate(spec) => spec.entity_id,
    };
    let kind = entity_kind(state, entity, work)?;
    match intent {
        EntityMutationIntent::UpdateFields(spec) => project_fields(kind, &spec.fields, touches),
        EntityMutationIntent::ApplyAspectPatch(_) => {
            unreachable!("candidate dispatch owns whole-struct state")
        }
        EntityMutationIntent::Replace(_) => {
            touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
        }
        EntityMutationIntent::Delete(_) => {
            touches.insert(ValidatedMutationTouch::DeleteEntity { kind });
        }
        EntityMutationIntent::Revalidate(_) => {
            touches.insert(ValidatedMutationTouch::RevalidateEntity { kind });
        }
    }
    Ok(())
}

fn project_relation(
    intent: &RelationMutationIntent,
    state: &crate::storage::overlay::WorkingState,
    touches: &mut super::collection::TouchCollection<'_, '_>,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<(), ValidatedMutationTouchProjectionError> {
    match intent {
        RelationMutationIntent::UpdateEndpoints(spec) => {
            touches.insert(ValidatedMutationTouch::UnlinkRelation { kind: spec.kind_id });
            touches.insert(ValidatedMutationTouch::LinkRelation { kind: spec.kind_id });
        }
        RelationMutationIntent::ApplyAspectPatch(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation { kind });
        }
        RelationMutationIntent::Revalidate(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::RevalidateRelation { kind });
        }
        RelationMutationIntent::Delete(spec) => {
            let kind = relation_kind(state, spec.relation_id, work)?;
            touches.insert(ValidatedMutationTouch::UnlinkRelation { kind });
        }
    }
    Ok(())
}

fn project_fields(
    kind: KindId,
    fields: &AspectFieldPatch,
    touches: &mut super::collection::TouchCollection<'_, '_>,
) {
    for locator in fields.locators() {
        if touches.failure.is_some() {
            break;
        }
        touches.field(kind, locator);
    }
}

fn project_entity_aspect_patch(
    kind: KindId,
    patch: &worth_foundational::facade::PortableRecordAspectPatch,
    touches: &mut super::collection::TouchCollection<'_, '_>,
) {
    for operation in patch.operations() {
        if touches.failure.is_some() {
            break;
        }
        match operation {
            PortableAspectPatchOperation::PatchFields {
                basis,
                selected_fields,
                ..
            } => {
                for field in selected_fields {
                    if touches.failure.is_some() {
                        break;
                    }
                    touches.selected_field(kind, basis, field);
                }
            }
            PortableAspectPatchOperation::SetWhole { .. }
            | PortableAspectPatchOperation::ClearWhole { .. } => {
                touches.insert(ValidatedMutationTouch::UnrepresentableEntityMutation { kind });
            }
        }
    }
}

fn project_relation_fields(
    kind: KindId,
    fields: &AspectFieldPatch,
    touches: &mut super::collection::TouchCollection<'_, '_>,
) {
    if fields.locators().next().is_some() {
        touches.insert(ValidatedMutationTouch::UnrepresentableRelationMutation { kind });
    }
}

fn entity_kind(
    state: &crate::storage::overlay::WorkingState,
    entity: EntityId,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<KindId, ValidatedMutationTouchProjectionError> {
    work.owner_state_lookups += 1;
    state
        .get_current_partition(entity.partition_id)
        .and_then(|partition| partition.entity_arena().get(&entity))
        .and_then(|record| record.kind_id())
        .ok_or(ValidatedMutationTouchProjectionError::MissingEntityKind(
            entity,
        ))
}

fn relation_kind(
    state: &crate::storage::overlay::WorkingState,
    relation: RelationId,
    work: &mut ValidatedMutationTouchProjectionWork,
) -> Result<KindId, ValidatedMutationTouchProjectionError> {
    work.owner_state_lookups += 1;
    state
        .get_current_partition(relation.partition_id)
        .and_then(|partition| partition.relation_arena().get(&relation))
        .and_then(|record| record.kind_id())
        .ok_or(ValidatedMutationTouchProjectionError::MissingRelationKind(
            relation,
        ))
}
