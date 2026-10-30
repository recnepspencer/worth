#[global_allocator]
static ALLOCATOR: tracking_allocator::Allocator<std::alloc::System> =
    tracking_allocator::Allocator::system();

#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/support/blob_crash/mod.rs"]
pub(crate) mod blob_crash;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/support/blob_expiry_recovery_negative/mod.rs"]
pub(crate) mod blob_expiry_recovery_negative;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/support/blob_reclaim_released_crash/mod.rs"]
pub(crate) mod blob_reclaim_released_crash;
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/cost/mod.rs"]
pub(crate) mod cost;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/support/layout_dedupe_collision/mod.rs"]
pub(crate) mod layout_dedupe_collision;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/support/physical_blob_journeys/mod.rs"]
pub(crate) mod physical_blob_journeys;
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/process/mod.rs"]
pub(crate) mod process;
#[path = "worth_store_court_recovery_runtime_replication_store_physical_blob_journeys/store/mod.rs"]
pub(crate) mod store;
