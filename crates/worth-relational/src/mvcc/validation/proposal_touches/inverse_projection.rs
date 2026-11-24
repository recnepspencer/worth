//! Project create attribution from the Native inverse and its allocation bindings.
use super::{collection, ValidatedMutationTouchProjectionError as Error};
use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::intents::mutation_intent::{CreateIntent, MutationIntent};
use crate::transactions::data::outcomes::{
    created_entity_bindings::CommitCreatedEntityBindings,
    created_relation_bindings::CommitCreatedRelationBindings,
};
use crate::transactions::data::primitives::{CreatedEntityRef, CreatedRelationRef};

pub(super) struct InverseProjection<'a, Anchors> {
    pub(super) created_entities: &'a CommitCreatedEntityBindings,
    pub(super) created_relations: &'a CommitCreatedRelationBindings,
    pub(super) non_anchor_entities: Anchors,
    pub(super) relations: &'a [(RelationId, CreatedRelationRef)],
}

pub(super) fn owns_create<'a>(
    inputs: InverseProjection<'a, impl FnOnce() -> &'a [(EntityId, CreatedEntityRef)]>,
    intent: &MutationIntent,
    checkpoint: &mut dyn FnMut(u64) -> Result<(), Error>,
) -> Result<bool, Error> {
    match intent {
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            let created = inputs.created_entities;
            let target = CreatedEntityRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
            };
            checkpoint(
                created
                    .resolve_work_bound(&target)
                    .ok_or_else(collection::capacity)?,
            )?;
            let id = created.resolve(&target);
            // Every anchor remains authored by its own inverse Create.
            for (_, target) in (inputs.non_anchor_entities)() {
                checkpoint(
                    created
                        .resolve_work_bound(target)
                        .ok_or_else(collection::capacity)?,
                )?;
                if id.is_some() && id == created.resolve(target) {
                    return Ok(true);
                }
            }
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            let created = inputs.created_relations;
            let target = CreatedRelationRef {
                partition_id: spec.partition_id,
                kind_id: spec.kind_id,
                client_key: spec.client_key.clone(),
                source: spec.source.clone(),
                target: spec.target.clone(),
            };
            checkpoint(
                created
                    .resolve_work_bound(&target)
                    .ok_or_else(collection::capacity)?,
            )?;
            let id = created.resolve(&target);
            for (_, target) in inputs.relations {
                checkpoint(
                    created
                        .resolve_work_bound(target)
                        .ok_or_else(collection::capacity)?,
                )?;
                if id.is_some() && id == created.resolve(target) {
                    return Ok(true);
                }
            }
        }
        _ => {}
    }
    Ok(false)
}
