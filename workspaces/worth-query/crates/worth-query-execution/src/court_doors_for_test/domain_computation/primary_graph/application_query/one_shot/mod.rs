//! Doors under `domain_computation::primary_graph::application_query::one_shot`.
#[path = "denial/mod.rs"]
pub mod denial;
pub use crate::domain_computation::primary_graph::application_query::one_shot::map_authorized_read_denial;

pub use crate::domain_computation::primary_graph::application_query::one_shot::WorthQueryAdmittedOneShotStop;
