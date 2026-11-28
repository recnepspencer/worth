use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::symbols::data::client_key::ClientKey;
use crate::contracts::symbols::data::string_interner::StringInterner;
use crate::contracts::symbols::data::symbol::Symbol;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, MutationIntent, RelationMutationIntent,
};
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::EntityReference;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::{
    collect_created_entity_refs,
    record_lookup::{entity_exists_in_state, require_live_target},
};

/// Rejects references outside the selected branch before client-key
/// normalization can mutate runtime-owned symbol state.
pub fn validate_branch_locality(
    state: &impl CurrentPartitionAccess,
    batches: &[crate::contracts::transactions::data::primitives::WorkerIntentBatch],
    interner: &crate::contracts::symbols::data::string_interner::StringInterner,
) -> Result<(), CommitConflict> {
    let created_entities =
        collect_created_entity_refs(batches.iter().flat_map(|batch| batch.intents.iter()))
            .into_iter()
            .map(|created| BranchLocalCreatedEntityRef::new(&created, interner))
            .collect();
    for intent in batches.iter().flat_map(|batch| &batch.intents) {
        if matches!(
            intent,
            MutationIntent::Entity(_) | MutationIntent::Relation(_)
        ) && let Some(target) = intent.existing_record_target()
        {
            require_live_target(state, target, "transaction branch-locality preflight")?;
        }
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
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                validate_endpoint(state, &created_entities, interner, &spec.source, "source")?;
                validate_endpoint(state, &created_entities, interner, &spec.target, "target")?;
            }
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::EntityAspects(_))
            | MutationIntent::Create(CreateIntent::BulkEntities(_))
            | MutationIntent::Materialization(_)
            | MutationIntent::Entity(_)
            | MutationIntent::Relation(_) => {}
        }
    }
    Ok(())
}

fn validate_endpoint(
    state: &impl CurrentPartitionAccess,
    created_entities: &std::collections::BTreeSet<BranchLocalCreatedEntityRef>,
    interner: &crate::contracts::symbols::data::string_interner::StringInterner,
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
        created: &crate::contracts::transactions::data::primitives::CreatedEntityRef,
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
                    .expect(/* Proof: Read-only resolution accepts all textual keys; only an unknown Symbol reaches this error branch, so this key has the Symbol variant. */ "only an unknown symbol can fail read-only resolution"),
            ),
        }
    }
}
