//! Descriptive topology changes consumed by canonical and installed readers.
use crate::identity::data::{EntityId, RelationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdjacencyDeltaKind {
    Created { source: EntityId, target: EntityId },
    Deleted { source: EntityId, target: EntityId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdjacencyDelta {
    pub(crate) relation_id: RelationId,
    pub(crate) kind_id: crate::identity::data::KindId,
    pub(crate) kind: AdjacencyDeltaKind,
}
