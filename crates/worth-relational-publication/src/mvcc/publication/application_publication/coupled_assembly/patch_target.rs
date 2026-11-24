//! Record addressed by one entity field or aspect patch.
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn patch_entity(intent: &MutationIntent) -> Option<EntityId> {
    match intent {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(p)) => Some(p.entity_id),
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(p)) => Some(p.entity_id),
        _ => None,
    }
}
