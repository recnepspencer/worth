mod backend;
mod builder;
mod conditional_installation;
mod domain_package_installation;
mod equivalence_report;
mod error;
mod installation_generation;
mod prepared_seed;
mod product_bridge;
mod product_world_resources;
mod runtime_installation;
mod schema;
mod seed;
mod support_profile;
mod workspace_build;
mod workspace_control;

pub use builder::{in_memory_test_runtime, WorthQueryInMemoryTestRuntimeBuilder};
pub use equivalence_report::{
    compare_test_backend_write_receipts, WorthQueryTestBackendEquivalenceReport,
    WorthQueryTestBackendEquivalenceRow,
};
pub use error::{WorthQueryTestBackendError, WorthQueryTestBackendErrorKind};
pub use installation_generation::advance_test_workspace_domain_installation_generation;
pub use product_world_resources::in_memory_test_product_world_resources;
pub use schema::WorthQueryTestBackendSchema;
pub use seed::{WorthQueryTestSeedReceipt, WorthQueryTestSeedRow};
pub use workspace_control::WorthQueryControlledTestWorkspace;

#[cfg(test)]
mod contract_fixtures;
#[cfg(test)]
pub(crate) use contract_fixtures::task_schema;
#[cfg(test)]
mod schema_tests;
#[cfg(test)]
mod support_profile_tests;
#[cfg(test)]
mod workspace_behavior_tests;
