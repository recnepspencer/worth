//! Three concrete observation owners, with no raw shared-storage escape.
mod entity_ids;
pub use entity_ids::InvariantEntityIds;
mod relation_ids;
pub use relation_ids::InvariantRelationIds;
mod partition_ids;
pub use partition_ids::InvariantPartitionIds;
