//! Doors under `domain_computation::primary_graph::provider::application_attempt_state`.
#[path = "commit_preparation/mod.rs"]
pub mod commit_preparation;
/// Whether a mid-envelope comparison's failure stops the edit (PR C r4).
pub use crate::domain_computation::primary_graph::provider::application_attempt_state::candidate_read_support::is_hard_comparison_stop as comparison_stops_hard_for_test;
