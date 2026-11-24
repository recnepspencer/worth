//! Record addressed by one entity field or aspect patch.
use crate::identity::data::EntityId;
use crate::transactions::data::intents::mutation_intent::{EntityMutationIntent, MutationIntent};

pub(super) fn patch_entity(intent: &MutationIntent) -> Option<EntityId> {
    match intent {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(p)) => Some(p.entity_id),
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(p)) => Some(p.entity_id),
        _ => None,
    }
}
