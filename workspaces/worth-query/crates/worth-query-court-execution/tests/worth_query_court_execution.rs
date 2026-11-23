#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Query execution proofs grouped by their product frontier.
#![recursion_limit = "256"]
mod compile_fail;
mod cost;
#[path = "worth_query_court_execution/domain_computation/mod.rs"]
pub mod domain_computation;
mod naming_bounded_depth;
mod naming_discriminator_agreement;
mod naming_execution_birth_wiring;
mod naming_execution_fixture;
mod naming_posture_agreement;
mod naming_reorder_and_collision;
mod naming_resolution_agreement;

mod query_obligations;
