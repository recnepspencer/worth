use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::primitives::TransactionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackEffect {
    RestoredEntity(EntityId),
    RestoredRelation(RelationId),
    DiscardedEntityCreation,
    DiscardedRelationCreation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RollbackSummary {
    pub restored_entity_count: usize,
    pub restored_relation_count: usize,
    pub discarded_entity_creation_count: usize,
    pub discarded_relation_creation_count: usize,
}

impl RollbackSummary {
    pub fn from_effects(effects: &[RollbackEffect]) -> Self {
        let mut summary = Self::default();

        for effect in effects {
            match effect {
                RollbackEffect::RestoredEntity(_) => {
                    summary.restored_entity_count += 1;
                }
                RollbackEffect::RestoredRelation(_) => {
                    summary.restored_relation_count += 1;
                }
                RollbackEffect::DiscardedEntityCreation => {
                    summary.discarded_entity_creation_count += 1;
                }
                RollbackEffect::DiscardedRelationCreation => {
                    summary.discarded_relation_creation_count += 1;
                }
            }
        }

        summary
    }

    pub fn total_effect_count(&self) -> usize {
        self.restored_entity_count
            + self.restored_relation_count
            + self.discarded_entity_creation_count
            + self.discarded_relation_creation_count
    }

    pub fn restored_record_count(&self) -> usize {
        self.restored_entity_count + self.restored_relation_count
    }

    pub fn discarded_creation_count(&self) -> usize {
        self.discarded_entity_creation_count + self.discarded_relation_creation_count
    }

    pub fn has_restored_entity(&self) -> bool {
        self.restored_entity_count > 0
    }

    pub fn has_discarded_entity_creation(&self) -> bool {
        self.discarded_entity_creation_count > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackOutcome {
    pub transaction_id: TransactionId,
    pub summary: RollbackSummary,
    pub effects: Vec<RollbackEffect>,
}

impl RollbackOutcome {
    pub fn summary(&self) -> &RollbackSummary {
        &self.summary
    }

    pub fn effects(&self) -> &[RollbackEffect] {
        &self.effects
    }

    pub fn effect_count(&self) -> usize {
        self.summary.total_effect_count()
    }

    pub fn has_effects(&self) -> bool {
        self.effect_count() > 0
    }
}
