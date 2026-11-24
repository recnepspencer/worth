//! Descriptive topology changes consumed by canonical and installed readers.
use crate::identity::data::{EntityId, RelationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjacencyDeltaKind {
    Created { source: EntityId, target: EntityId },
    Deleted { source: EntityId, target: EntityId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdjacencyDelta {
    pub relation_id: RelationId,
    pub kind_id: crate::identity::data::KindId,
    pub kind: AdjacencyDeltaKind,
}
