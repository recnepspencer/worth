//! Doors under `domain_computation::primary_graph::application_contribution::request_execution`.
#[path = "advancement/mod.rs"]
pub mod advancement;
#[path = "test_placement/mod.rs"]
pub mod test_placement;
pub use crate::domain_computation::primary_graph::request_execution::QueryMemoryReservation;
pub use crate::domain_computation::primary_graph::request_execution::QueryRequestExecution;
pub use crate::domain_computation::primary_graph::request_execution::Resource;
