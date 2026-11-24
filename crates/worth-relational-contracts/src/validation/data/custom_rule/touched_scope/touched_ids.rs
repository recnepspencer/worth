//! Three concrete observation owners, with no raw shared-storage escape.
pub mod entity_ids;
pub use entity_ids::InvariantEntityIds;
pub mod relation_ids;
pub use relation_ids::InvariantRelationIds;
pub mod partition_ids;
pub use partition_ids::InvariantPartitionIds;
