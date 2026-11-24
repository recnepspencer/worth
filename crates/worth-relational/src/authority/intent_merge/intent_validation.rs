use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::intents::mutation_intent::{
    CreateIntent, MutationIntent, RelationMutationIntent,
};
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::primitives::CreatedEntityRef;
use std::collections::BTreeSet;

use super::entity_validation::validate_entity_intent;
use super::relation_validation::validate_relation_intent;

pub(crate) fn validate_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    candidates_scanned: &mut usize,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::identity::data::RelationId>,
    intent: &MutationIntent,
) -> Result<(), CommitConflict> {
    validate_entity_intent(state, schema, intent)?;
    validate_relation_intent(
        state,
        schema,
        default_cross_context_policy,
        candidates_scanned,
        created_entities,
        deleted_relations,
        intent,
    )?;
    Ok(())
}

pub(crate) fn collect_deleted_relation_ids<'a>(
    intents: impl IntoIterator<Item = &'a MutationIntent>,
) -> BTreeSet<crate::identity::data::RelationId> {
    intents
        .into_iter()
        .filter_map(|intent| match intent {
            MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                Some(spec.relation_id)
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn collect_created_entity_refs<'a>(
    intents: impl IntoIterator<Item = &'a MutationIntent>,
) -> BTreeSet<CreatedEntityRef> {
    let mut created = BTreeSet::new();
    for intent in intents {
        match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                created.insert(CreatedEntityRef {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                });
            }
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                created.insert(CreatedEntityRef {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                });
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                for client_key in &spec.client_keys {
                    created.insert(CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                    });
                }
            }
            MutationIntent::Create(CreateIntent::Relation(_))
            | MutationIntent::Create(CreateIntent::RelationAspects(_))
            | MutationIntent::Create(CreateIntent::BulkRelations(_))
            | MutationIntent::Entity(_)
            | MutationIntent::Relation(_)
            | MutationIntent::Materialization(_) => {}
        }
    }
    created
}
