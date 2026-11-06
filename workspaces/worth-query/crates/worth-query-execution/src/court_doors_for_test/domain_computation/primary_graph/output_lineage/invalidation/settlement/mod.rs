//! Doors under `domain_computation::primary_graph::output_lineage::invalidation::settlement`.
#[path = "posting_bindings/mod.rs"]
pub mod posting_bindings;
#[path = "posting_projection/mod.rs"]
pub mod posting_projection;
#[path = "posting_removal/mod.rs"]
pub mod posting_removal;
#[path = "postings/mod.rs"]
pub mod postings;
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::settlement::SettlementRegistration;
