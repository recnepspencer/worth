use crate::capabilities::StorageRead;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::{ClientKey, StringInterner, Symbol};
use crate::transactions::data::{
    CommitConflict, ConflictClass, CreateIntent, EntityMutationIntent, EntityReference,
    ExistingRecordTarget, MutationIntent, RelationMutationIntent,
};

use super::{
    collect_created_entity_refs,
    record_lookup::{entity_exists_in_state, relation_exists_in_state},
};

/// Rejects references outside the selected branch before client-key
/// normalization can mutate runtime-owned symbol state.
pub(crate) fn validate_branch_locality(
    state: &impl StorageRead,
    batches: &[crate::transactions::data::WorkerIntentBatch],
    interner: &crate::symbols::data::StringInterner,
) -> Result<(), CommitConflict> {
    let created_entities =
        collect_created_entity_refs(batches.iter().flat_map(|batch| batch.intents.iter()))
            .into_iter()
            .map(|created| BranchLocalCreatedEntityRef::new(&created, interner))
            .collect();
    for intent in batches.iter().flat_map(|batch| &batch.intents) {
        match intent {
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                validate_endpoint(state, &created_entities, interner, &spec.source, "source")?;
                validate_endpoint(state, &created_entities, interner, &spec.target, "target")?;
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                validate_endpoint(state, &created_entities, interner, &spec.source, "source")?;
                validate_endpoint(state, &created_entities, interner, &spec.target, "target")?;
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                for (source, target) in &spec.endpoints {
                    validate_endpoint(state, &created_entities, interner, source, "source")?;
                    validate_endpoint(state, &created_entities, interner, target, "target")?;
                }
            }
            MutationIntent::Entity(entity) => {
                let entity_id = match entity {
                    EntityMutationIntent::UpdateFields(spec) => spec.entity_id,
                    EntityMutationIntent::ApplyAspectPatch(spec) => spec.entity_id,
                    EntityMutationIntent::Replace(spec) => spec.entity_id,
                    EntityMutationIntent::Delete(spec) => spec.entity_id,
                };
                if !entity_exists_in_state(state, entity_id) {
                    return Err(CommitConflict::new(ConflictClass::StaleTarget {
                        target: ExistingRecordTarget::Entity(entity_id),
                        context: "transaction branch-locality preflight".to_owned(),
                    }));
                }
            }
            MutationIntent::Relation(relation) => {
                let relation_id = match relation {
                    RelationMutationIntent::UpdateEndpoints(spec) => spec.relation_id,
                    RelationMutationIntent::ApplyAspectPatch(spec) => spec.relation_id,
                    RelationMutationIntent::Delete(spec) => spec.relation_id,
                };
                if !relation_exists_in_state(state, relation_id) {
                    return Err(CommitConflict::new(ConflictClass::StaleTarget {
                        target: ExistingRecordTarget::Relation(relation_id),
                        context: "transaction branch-locality preflight".to_owned(),
                    }));
                }
                if let RelationMutationIntent::UpdateEndpoints(spec) = relation {
                    validate_endpoint(state, &created_entities, interner, &spec.source, "source")?;
                    validate_endpoint(state, &created_entities, interner, &spec.target, "target")?;
                }
            }
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::EntityAspects(_))
            | MutationIntent::Create(CreateIntent::BulkEntities(_))
            | MutationIntent::Materialization(_) => {}
        }
    }
    Ok(())
}

fn validate_endpoint(
    state: &impl StorageRead,
    created_entities: &std::collections::BTreeSet<BranchLocalCreatedEntityRef>,
    interner: &crate::symbols::data::StringInterner,
    endpoint: &EntityReference,
    label: &str,
) -> Result<(), CommitConflict> {
    let admitted = match endpoint {
        EntityReference::Existing(entity_id) => entity_exists_in_state(state, *entity_id),
        EntityReference::Created(created) => {
            created_entities.contains(&BranchLocalCreatedEntityRef::new(created, interner))
        }
    };
    if admitted {
        Ok(())
    } else {
        Err(CommitConflict::new(ConflictClass::InvalidRelationEndpoint {
            detail: format!(
                "transaction {label} endpoint must be live on the selected branch or created by the same transaction"
            ),
        }))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct BranchLocalCreatedEntityRef {
    partition_id: PartitionId,
    kind_id: KindId,
    client_key: BranchLocalClientKey,
}

impl BranchLocalCreatedEntityRef {
    fn new(
        created: &crate::transactions::data::CreatedEntityRef,
        interner: &StringInterner,
    ) -> Self {
        Self {
            partition_id: created.partition_id,
            kind_id: created.kind_id,
            client_key: BranchLocalClientKey::new(&created.client_key, interner),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum BranchLocalClientKey {
    Text(String),
    UnresolvedSymbol(Symbol),
}

impl BranchLocalClientKey {
    fn new(client_key: &ClientKey, interner: &StringInterner) -> Self {
        match client_key.resolve_with_interner(interner) {
            Some(text) => Self::Text(text.to_owned()),
            None => Self::UnresolvedSymbol(
                client_key
                    .as_symbol()
                    .expect("only an unknown symbol can fail read-only resolution"),
            ),
        }
    }
}
