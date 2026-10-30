#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[macro_use]
extern crate worth_store_testkit_store;

#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/support/bounded_physical_record_access_examples/mod.rs"]
pub(crate) mod bounded_physical_record_access_examples;
#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/compile_fail/mod.rs"]
pub(crate) mod compile_fail;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/support/physical_record_journeys/mod.rs"]
pub(crate) mod physical_record_journeys;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/support/physical_runtime_authority_ui/mod.rs"]
pub(crate) mod physical_runtime_authority_ui;
#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/process/mod.rs"]
pub(crate) mod process;

#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/cost/mod.rs"]
pub(crate) mod cost;
#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/supported_programs/mod.rs"]
mod supported_programs;

#[path = "worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/support/durability.rs"]
mod admitted_durability;
