mod canonical_durability;
mod configuration;
mod journal_registry;
mod observed_allocation;
mod record_chunk_world;
#[cfg(test)]
mod record_chunk_world_tests;
mod store_world;

#[cfg(feature = "recovery-runtime-fixtures")]
pub use canonical_durability::canonical_durable_wal_attempt_without_execution;
#[cfg(feature = "recovery-runtime-fixtures")]
pub use canonical_durability::canonical_rooted_mutation_without_acknowledgment;
pub use canonical_durability::{
    canonical_physical_batch_acknowledgment, canonical_physical_mutation_acknowledgment,
};
pub use configuration::SUCCESSOR_SCOPE_ALLOCATION_BYTES;
pub use observed_allocation::{observed_store_residency, PhysicalResidencyFixtureWorkload};
pub use record_chunk_world::PhysicalResidencyRecordWorldFailure;
pub use store_world::{
    PhysicalResidencyStoreWorld, PhysicalResidencyStoreWorldConstructionFailure,
};
