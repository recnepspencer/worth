pub mod primitives;
pub mod record_ids;

pub use primitives::{
    Generation, KindId, LineageId, LocalSlot, PartitionId, StructuralFingerprint, VersionBound,
    VersionId,
};
pub use record_ids::{
    EntityDomain, EntityId, EntityStorageId, RecordId, RelationDomain, RelationId,
    RelationStorageId,
};
