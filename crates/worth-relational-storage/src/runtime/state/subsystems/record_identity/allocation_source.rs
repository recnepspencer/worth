//! The intent occurrence that allocates a record, independent of how a
//! coupled suffix lowers created endpoints for apply. Live coupled allocation
//! runs member by member; recovery re-applies the envelope's one merged plan,
//! so the live evidence is reordered to that plan by these sources (H0).
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllocationSource {
    Entity {
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: ClientKey,
    },
    /// The replacement record of a `Replace` of this entity.
    Replacement(EntityId),
    Relation {
        partition_id: PartitionId,
        kind_id: KindId,
        client_key: ClientKey,
    },
}

impl AllocationSource {
    /// The records `intent` allocates, in its apply order (bulk rows in row
    /// order); empty for an intent that allocates nothing. O(rows).
    pub fn of(intent: &MutationIntent) -> Vec<Self> {
        let entity = |partition_id, kind_id, client_key: &ClientKey| Self::Entity {
            partition_id,
            kind_id,
            client_key: client_key.clone(),
        };
        let relation = |partition_id, kind_id, client_key: &ClientKey| Self::Relation {
            partition_id,
            kind_id,
            client_key: client_key.clone(),
        };
        match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                vec![entity(spec.partition_id, spec.kind_id, &spec.client_key)]
            }
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                vec![entity(spec.partition_id, spec.kind_id, &spec.client_key)]
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => spec
                .client_keys
                .iter()
                .map(|key| entity(spec.partition_id, spec.kind_id, key))
                .collect(),
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                vec![relation(spec.partition_id, spec.kind_id, &spec.client_key)]
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                vec![relation(spec.partition_id, spec.kind_id, &spec.client_key)]
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => spec
                .client_keys
                .iter()
                .map(|key| relation(spec.partition_id, spec.kind_id, key))
                .collect(),
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                vec![Self::Replacement(spec.entity_id)]
            }
            MutationIntent::Entity(_)
            | MutationIntent::Relation(_)
            | MutationIntent::Materialization(_) => Vec::new(),
        }
    }
}
