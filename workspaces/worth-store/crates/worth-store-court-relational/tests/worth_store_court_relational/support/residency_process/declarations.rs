//! Cold serving and interrupted cold readmission use the existing kill driver.
pub(crate) use {crate::durable_run, crate::killed_child, crate::residency_fixture::Fixture, crate::schema_transition, crate::workload};
pub(crate) use {std::fs, std::path::PathBuf};
pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_store_relational::RelationalStoreHome, worth_store_relational::RelationalStoreHomeRequest};

pub(crate) const ROLE: &str = "WORTH_RESIDENCY_CHILD";
pub(crate) const LOCATION: &str = "WORTH_RESIDENCY_LOCATION";
pub(crate) const MARKERS: &str = "WORTH_RESIDENCY_MARKERS";
pub(crate) const CHILD: &str = "residency_process::residency_child";