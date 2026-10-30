#[macro_use]
extern crate worth_store_testkit_store;

#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> =
    worth_store_testkit_store::allocation_probe::physical_record_journeys::ALLOCATOR;

#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/child_process/mod.rs"]
pub(crate) mod child_process;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/courtroom_evidence_support/mod.rs"]
pub(crate) mod courtroom_evidence_support;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/courtroom_oracle/mod.rs"]
pub(crate) mod courtroom_oracle;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/durability_admission/mod.rs"]
pub(crate) mod durability_admission;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/observer/mod.rs"]
pub(crate) mod observer;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/physical_record_journeys/mod.rs"]
pub(crate) mod physical_record_journeys;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/support/physical_work/mod.rs"]
pub(crate) mod physical_work;
#[path = "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission/process/mod.rs"]
pub(crate) mod process;
