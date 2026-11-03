//! Doors under `domain_computation::primary_graph::application_output_demand::registry::post_commit`.
#[path = "custody_reader/mod.rs"]
pub mod custody_reader;
#[path = "discovery/mod.rs"]
pub mod discovery;
#[path = "retirement_observation/mod.rs"]
pub mod retirement_observation;
#[path = "source_retirement/mod.rs"]
pub mod source_retirement;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::post_commit::sweep_contacts;
