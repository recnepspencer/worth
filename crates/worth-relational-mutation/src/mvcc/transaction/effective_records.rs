//! Cached transaction reads carry their exact authored prefix.
use super::relation_value::RelationalTransactionRelationValue;
use super::staged_mutations::RelationalTransactionStagedMutations;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::storage::data::read_records::EntityReadRecord;
use crate::contracts::transactions::data::intents::mutation_intent::{
    EntityMutationIntent, RelationMutationIntent,
};

#[derive(Debug, Clone, Default)]
pub struct EffectiveRecords {
    pub entities: im::OrdMap<EntityId, RecordProjection<EntityMutationIntent, EntityReadRecord>>,
    pub relations: im::OrdMap<
        RelationId,
        RecordProjection<RelationMutationIntent, RelationalTransactionRelationValue>,
    >,
}

#[derive(Debug, Clone)]
pub struct RecordProjection<Intent: Clone, Value> {
    pub effective: Option<Value>,
    pub staged: RelationalTransactionStagedMutations<Intent>,
    pub through_batches: usize,
    pub deleted: bool,
}
