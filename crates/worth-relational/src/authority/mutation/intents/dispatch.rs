use crate::transactions::data::{
    CommitConflict, CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
};

use super::{
    apply_entity_aspect_patch, apply_relation_aspect_patch, bulk_create_entities,
    bulk_create_relations, create_entity, create_entity_aspects, create_relation,
    create_relation_aspects, delete_entity, delete_relation, materialization, replace_entity,
    revalidate_entity, update_entity_fields, update_relation_endpoints,
};
use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::MutationWorkspace;

pub(crate) fn dispatch_intent(
    intent: &MutationIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => create_entity::apply(spec, workspace),
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            create_entity_aspects::apply(spec, workspace)
        }
        MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
            bulk_create_entities::apply(spec, workspace)
        }
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
            update_entity_fields::apply(spec, workspace)
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
            apply_entity_aspect_patch::apply(spec, workspace)
        }
        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
            replace_entity::apply(spec, workspace)
        }
        MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
            delete_entity::apply(spec, workspace)
        }
        MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
            revalidate_entity::apply(spec, workspace)
        }
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            create_relation::apply(spec, workspace)
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            create_relation_aspects::apply(spec, workspace)
        }
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            bulk_create_relations::apply(spec, workspace)
        }
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
            update_relation_endpoints::apply(spec, workspace)
        }
        MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
            apply_relation_aspect_patch::apply(spec, workspace)
        }
        MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
            delete_relation::apply(spec, workspace)
        }
        MutationIntent::Materialization(intent) => materialization::apply(intent, workspace),
    }
}
