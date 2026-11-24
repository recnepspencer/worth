//! Per-identity projection frontiers on one exact transaction basis.
use super::relation_value::RelationalTransactionRelationValue;
use super::staged_mutations::RelationalTransactionStagedMutations as Staged;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::storage::data::read_records::EntityReadRecord;
use crate::contracts::transactions::data::intents::mutation_intent::{
    EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use crate::contracts::transactions::data::primitives::WorkerIntentBatch;

#[derive(Debug, Clone, Default)]
pub struct EffectiveRecords {
    pub entity_history: im::OrdMap<EntityId, Staged<EntityMutationIntent>>,
    pub relation_history: im::OrdMap<RelationId, Staged<RelationMutationIntent>>,
    pub entities: im::OrdMap<EntityId, EntityProjection>,
    pub relations: im::OrdMap<RelationId, RelationProjection>,
}
#[derive(Debug, Clone)]
pub struct EntityProjection {
    pub effective: Option<EntityReadRecord>,
    pub applied: usize,
    pub deleted: bool,
}
#[derive(Debug, Clone)]
pub struct RelationProjection {
    pub effective: Option<RelationalTransactionRelationValue>,
    pub applied: usize,
    pub deleted: bool,
}
impl EffectiveRecords {
    pub fn stage(&mut self, batch: &WorkerIntentBatch) {
        for intent in &batch.intents {
            match intent {
                MutationIntent::Entity(intent) => {
                    use super::intent_locus::{entity_intent_locus, EntityIntentLocus};
                    let (EntityIntentLocus::Write(id) | EntityIntentLocus::Read(id)) =
                        entity_intent_locus(intent);
                    self.entity_history
                        .entry(id)
                        .or_default()
                        .push(intent.clone());
                }
                MutationIntent::Relation(intent) => {
                    use super::intent_locus::{relation_intent_locus, RelationIntentLocus};
                    let (RelationIntentLocus::Write(id) | RelationIntentLocus::Read(id)) =
                        relation_intent_locus(intent);
                    self.relation_history
                        .entry(id)
                        .or_default()
                        .push(intent.clone());
                }
                _ => {}
            }
        }
    }
}
