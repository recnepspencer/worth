//! Doors under `domain_computation::primary_graph::vocabulary::output_lineage`.
#[path = "commit_touches/mod.rs"]
pub mod commit_touches;
#[path = "derived/mod.rs"]
pub mod derived;
#[path = "mark_state/mod.rs"]
pub mod mark_state;
#[path = "semantic_source/mod.rs"]
pub mod semantic_source;
#[path = "settlement_identity/mod.rs"]
pub mod settlement_identity;
pub use crate::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;
