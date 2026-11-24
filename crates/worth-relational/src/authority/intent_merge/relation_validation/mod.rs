mod endpoint_admission;
mod endpoint_update_admission;
mod relation_creation_admission;
mod relation_identity_scan;
mod relation_target_admission;
mod schema_relation_admission;

use std::collections::BTreeSet;

use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::intents::mutation_intent::{
    CreateIntent, MutationIntent, RelationMutationIntent,
};
use crate::transactions::data::primitives::CreatedEntityRef;

pub(super) fn validate_relation_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    candidates_scanned: &mut usize,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::identity::data::RelationId>,
    intent: &MutationIntent,
) -> Result<(), crate::transactions::data::outcomes::commit_conflict::CommitConflict> {
    match intent {
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            relation_creation_admission::validate_relation_creation_intent(
                state,
                schema,
                default_cross_context_policy,
                candidates_scanned,
                created_entities,
                deleted_relations,
                spec,
            )
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            relation_creation_admission::validate_relation_creation(
                state,
                schema,
                default_cross_context_policy,
                candidates_scanned,
                created_entities,
                deleted_relations,
                spec.partition_id,
                spec.kind_id,
                &spec.source,
                &spec.target,
            )
        }
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            relation_creation_admission::validate_bulk_relation_creation_intent(
                state,
                schema,
                default_cross_context_policy,
                candidates_scanned,
                created_entities,
                deleted_relations,
                spec.partition_id,
                spec.kind_id,
                &spec.endpoints,
            )
        }
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
            endpoint_update_admission::validate_relation_endpoint_update_intent(
                state,
                schema,
                default_cross_context_policy,
                candidates_scanned,
                created_entities,
                spec,
            )
        }
        MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
            relation_target_admission::validate_existing_relation_target(state, spec.relation_id)
        }
        MutationIntent::Relation(RelationMutationIntent::Revalidate(spec)) => {
            relation_target_admission::validate_existing_relation_target(state, spec.relation_id)
        }
        MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
            relation_target_admission::validate_existing_relation_target(state, spec.relation_id)
        }
        MutationIntent::Create(CreateIntent::Entity(_))
        | MutationIntent::Create(CreateIntent::EntityAspects(_))
        | MutationIntent::Create(CreateIntent::BulkEntities(_))
        | MutationIntent::Entity(_)
        | MutationIntent::Materialization(_) => Ok(()),
    }
}
