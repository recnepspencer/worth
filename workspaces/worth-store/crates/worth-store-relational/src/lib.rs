//! Relational native section, journal placement and cold demand reads.
//! Physical lifecycle and checkpoint selection belong to Store Application.
mod cold_source;
mod journal_medium;
mod recovered_history;
mod recovered_transitions;
mod resolution;
mod section;
mod transition_resolution;

pub mod facade;
pub use facade::*;
