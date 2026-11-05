//! Doors under `domain_computation::primary_graph::application_query::one_shot`.
#[path = "correction_disclosure/mod.rs"]
pub mod correction_disclosure;
#[path = "court_doors_for_test/mod.rs"]
pub mod court_doors_for_test;
#[path = "denial/mod.rs"]
pub mod denial;
pub use crate::domain_computation::primary_graph::application_query::one_shot::map_authorized_read_denial;
