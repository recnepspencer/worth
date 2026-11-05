//! Doors under `domain_computation::primary_graph::application_query::observed_source`.
#[path = "fact_conversion/mod.rs"]
pub mod fact_conversion;
#[path = "prepared_expectation/mod.rs"]
pub mod prepared_expectation;
#[path = "root_selection/mod.rs"]
pub mod root_selection;
#[path = "source_identity/mod.rs"]
pub mod source_identity;
pub use crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint;
