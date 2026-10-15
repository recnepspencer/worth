//! Consolidated snapshot baseline, dependency, and facade-surface contracts.

mod query_ordinary_entry_fixture;

#[path = "snapshot_contracts/baseline.rs"]
mod baseline;
#[path = "snapshot_contracts/dependency_classes.rs"]
mod dependency_classes;
#[path = "snapshot_contracts/surface.rs"]
mod surface;
