//! Per-identity projection frontiers on one exact transaction basis.
use super::read_projection::RelationalTransactionRelationValue;
use super::staged_mutations::RelationalTransactionStagedMutations as Staged;
use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::EntityReadRecord;
use crate::transactions::data::{
    EntityMutationIntent, MutationIntent, RelationMutationIntent, WorkerIntentBatch,
};

#[derive(Debug, Clone, Default)]
pub(super) struct EffectiveRecords {
    pub(super) entity_history: im::OrdMap<EntityId, Staged<EntityMutationIntent>>,
    pub(super) relation_history: im::OrdMap<RelationId, Staged<RelationMutationIntent>>,
    pub(super) entities: im::OrdMap<EntityId, EntityProjection>,
    pub(super) relations: im::OrdMap<RelationId, RelationProjection>,
}
#[derive(Debug, Clone)]
pub(super) struct EntityProjection {
    pub(super) effective: Option<EntityReadRecord>,
    pub(super) applied: usize,
    pub(super) deleted: bool,
}
#[derive(Debug, Clone)]
pub(super) struct RelationProjection {
    pub(super) effective: Option<RelationalTransactionRelationValue>,
    pub(super) applied: usize,
    pub(super) deleted: bool,
}
impl EffectiveRecords {
    pub(super) fn stage(&mut self, batch: &WorkerIntentBatch) {
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
