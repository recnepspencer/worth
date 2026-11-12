use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    TransactionCommitError,
};

use super::{
    apply_entity_aspect_patch, apply_relation_aspect_patch, bulk_create_entities,
    bulk_create_relations, create_entity, create_entity_aspects, create_relation,
    create_relation_aspects, delete_entity, delete_relation, materialization, replace_entity,
    revalidate_entity, revalidate_relation, update_entity_fields, update_relation_endpoints,
};
use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::MutationWorkspace;

pub(crate) fn dispatch_intent<'a>(
    intent: &'a MutationIntent,
    workspace: &mut MutationWorkspace<'a>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<MutationOutcome, TransactionCommitError> {
    match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => {
            workspace.run_owned_mutation(|workspace| create_entity::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            workspace.run_owned_mutation(|workspace| create_entity_aspects::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
            bulk_create_entities::apply(spec, workspace, lease)
        }
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
            workspace.run_owned_mutation(|workspace| update_entity_fields::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => workspace
            .run_owned_mutation(|workspace| apply_entity_aspect_patch::apply(spec, workspace)),
        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
            workspace.run_owned_mutation(|workspace| replace_entity::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
            workspace.run_owned_mutation(|workspace| delete_entity::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
            workspace.run_owned_mutation(|workspace| revalidate_entity::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            workspace.run_owned_mutation(|workspace| create_relation::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => workspace
            .run_owned_mutation(|workspace| create_relation_aspects::apply(spec, workspace)),
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            bulk_create_relations::apply(spec, workspace, lease)
        }
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => workspace
            .run_owned_mutation(|workspace| update_relation_endpoints::apply(spec, workspace)),
        MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => workspace
            .run_owned_mutation(|workspace| apply_relation_aspect_patch::apply(spec, workspace)),
        MutationIntent::Relation(RelationMutationIntent::Revalidate(spec)) => {
            workspace.run_owned_mutation(|workspace| revalidate_relation::apply(spec, workspace))
        }
        MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
            workspace.run_owned_mutation(|workspace| delete_relation::apply(spec, workspace))
        }
        MutationIntent::Materialization(intent) => {
            workspace.run_owned_mutation(|workspace| materialization::apply(intent, workspace))
        }
    }
}
