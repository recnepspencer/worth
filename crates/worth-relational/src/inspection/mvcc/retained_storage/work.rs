//! Work performed by one explicit owner census, never publication accounting.

/// Actual traversal counts and component elapsed times for a retained census.
/// Times are diagnostic samples, not deterministic acceptance thresholds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelationalRetainedStorageWork {
    pub roots: u64,
    pub region_entries: u64,
    pub payload_allocation_attempts: u64,
    pub payload_new_allocations: u64,
    pub root_allocation_rows: u64,
    pub schema_inventory_computations: u64,
    pub excluded_partition_walks: u64,
    pub catalog_artifacts: u64,
    pub partition_walk_us: u128,
    pub root_inventory_us: u128,
    /// Union traversal including the separately reported partition visits.
    pub root_union_us: u128,
    pub excluded_walk_us: u128,
    pub catalog_inventory_us: u128,
}
