//! Downstream journeys through the ordinary declarative and installed-domain facades.
pub use worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy;

// Downstream journey courts share this non-product support module.
pub use worth_query_testkit_installed_operation_fixture::facade::query::support;

#[path = "causal_inspection_public_dx.rs"]
pub mod causal_inspection_public_dx;
#[path = "declarative_history_comparison_public_dx.rs"]
pub mod declarative_history_comparison_public_dx;
#[path = "declarative_inspection_public_dx.rs"]
pub mod declarative_inspection_public_dx;
#[path = "declarative_live_public_dx.rs"]
pub mod declarative_live_public_dx;
#[path = "declarative_product_boundary_certification.rs"]
pub mod declarative_product_boundary_certification;
#[path = "declarative_read_public_dx.rs"]
pub mod declarative_read_public_dx;
#[path = "declarative_workflow_public_dx.rs"]
pub mod declarative_workflow_public_dx;
#[path = "installed_domain_facade_extension.rs"]
pub mod installed_domain_facade_extension;
#[path = "native_aspect_mutation_public_dx.rs"]
pub mod native_aspect_mutation_public_dx;
