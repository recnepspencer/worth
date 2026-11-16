#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Query execution proofs grouped by their product frontier.
#![recursion_limit = "256"]
mod compile_fail;
mod cost;
#[path = "worth_query_court_execution/domain_computation/mod.rs"]
pub mod domain_computation;

mod query_obligations;
