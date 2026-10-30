//! Canonical Query fixture family.
#![recursion_limit = "512"]
#[allow(
    dead_code,
    reason = "Shared private transport supports typed fields used by other closed testkits."
)]
#[path = "../../../../../scripts/test_cost/setup_observation.rs"]
mod setup_observation;

pub mod facade;
