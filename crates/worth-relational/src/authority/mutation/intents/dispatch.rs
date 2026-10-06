use crate::transactions::data::{
    CommitConflict, CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    TransactionCommitError,
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
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<MutationOutcome, TransactionCommitError> {
    match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => {
            from_conflict(create_entity::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
            from_conflict(create_entity_aspects::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
            bulk_create_entities::apply(spec, workspace, lease)
        }
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
            from_conflict(update_entity_fields::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
            from_conflict(apply_entity_aspect_patch::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
            from_conflict(replace_entity::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
            from_conflict(delete_entity::apply(spec, workspace))
        }
        MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
            from_conflict(revalidate_entity::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            from_conflict(create_relation::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            from_conflict(create_relation_aspects::apply(spec, workspace))
        }
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            bulk_create_relations::apply(spec, workspace, lease)
        }
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
            from_conflict(update_relation_endpoints::apply(spec, workspace))
        }
        MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
            from_conflict(apply_relation_aspect_patch::apply(spec, workspace))
        }
        MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
            from_conflict(delete_relation::apply(spec, workspace))
        }
        MutationIntent::Materialization(intent) => {
            from_conflict(materialization::apply(intent, workspace))
        }
    }
}

fn from_conflict(
    result: Result<MutationOutcome, CommitConflict>,
) -> Result<MutationOutcome, TransactionCommitError> {
    result.map_err(TransactionCommitError::conflict)
}
