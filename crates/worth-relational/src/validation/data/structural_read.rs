//! Structural record coordinates and read refusals.
use crate::identity::data::{EntityId, KindId, RelationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StructuralRelationRecord {
    pub relation_id: RelationId,
    pub kind_id: KindId,
    pub source: EntityId,
    pub target: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralReadError {
    WorkBudgetExceeded,
    OutsideDeclaredAccess,
    RecordUnavailable,
}
