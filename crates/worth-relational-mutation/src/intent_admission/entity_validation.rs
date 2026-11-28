use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent,
};
use crate::contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::contracts::transactions::data::primitives::ExistingRecordTarget;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::record_lookup::require_live_target;
use super::schema_conflicts::schema_error_to_commit_conflict;

pub(super) fn validate_entity_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    intent: &MutationIntent,
) -> Result<(), CommitConflict> {
    let schema_registry = schema;
    match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => schema_registry
            .resolve_entity(spec.kind_id)
            .map(|_| ())
            .map_err(schema_error_to_commit_conflict),
        MutationIntent::Create(CreateIntent::EntityAspects(spec)) => schema_registry
            .resolve_entity(spec.kind_id)
            .map(|_| ())
            .map_err(schema_error_to_commit_conflict),
        MutationIntent::Create(CreateIntent::BulkEntities(spec)) => schema_registry
            .resolve_entity(spec.kind_id)
            .map(|_| ())
            .map_err(schema_error_to_commit_conflict),
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
            validate_existing_entity_intent(state, schema, spec.entity_id, intent)
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
            validate_existing_entity_intent(state, schema, spec.entity_id, intent)
        }
        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
            validate_existing_entity_intent(state, schema, spec.entity_id, intent)
        }
        MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
            validate_existing_entity_intent(state, schema, spec.entity_id, intent)
        }
        MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
            validate_existing_entity_intent(state, schema, spec.entity_id, intent)
        }
        MutationIntent::Create(CreateIntent::Relation(_))
        | MutationIntent::Create(CreateIntent::RelationAspects(_))
        | MutationIntent::Create(CreateIntent::BulkRelations(_))
        | MutationIntent::Relation(_)
        | MutationIntent::Materialization(_) => Ok(()),
    }
}

fn validate_existing_entity_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    entity_id: crate::contracts::identity::data::record_ids::EntityId,
    intent: &MutationIntent,
) -> Result<(), CommitConflict> {
    let schema_registry = schema;
    require_live_target(
        state,
        ExistingRecordTarget::Entity(entity_id),
        "entity validation",
    )?;

    match intent {
        MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => schema_registry
            .resolve_entity(spec.replacement.kind_id)
            .map(|_| ())
            .map_err(schema_error_to_commit_conflict),
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(_)) => Ok(()),
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_)) => Ok(()),
        MutationIntent::Create(_)
        | MutationIntent::Entity(EntityMutationIntent::Delete(_))
        | MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
        | MutationIntent::Relation(_)
        | MutationIntent::Materialization(_) => Ok(()),
    }
}
