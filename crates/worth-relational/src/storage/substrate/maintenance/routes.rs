//! Semantic install-time routing coordinates, independent of catalog size.
use crate::identity::data::{KindId, PartitionId};
use crate::indexes::data::DerivedIndexId;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaintenanceRoute {
    Partition(PartitionId),
    EntityKind(KindId),
    RelationKind(KindId),
    Field {
        kind: KindId,
        aspect: AspectKey,
        path: CanonicalFieldPath,
    },
    Index(DerivedIndexId),
    Schema,
    Symbols,
}
