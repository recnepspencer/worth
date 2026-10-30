#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[macro_use]
extern crate worth_store_testkit_store;

#[path = "worth_store_court_store_replication_physical_blob_journeys/compile_fail/mod.rs"]
pub(crate) mod compile_fail;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_store_replication_physical_blob_journeys/support/physical_blob_journeys/mod.rs"]
pub(crate) mod physical_blob_journeys;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_store_replication_physical_blob_journeys/support/physical_record_journeys/mod.rs"]
pub(crate) mod physical_record_journeys;
#[path = "worth_store_court_store_replication_physical_blob_journeys/process/mod.rs"]
pub(crate) mod process;

#[path = "worth_store_court_store_replication_physical_blob_journeys/supported_programs/mod.rs"]
mod supported_programs;
