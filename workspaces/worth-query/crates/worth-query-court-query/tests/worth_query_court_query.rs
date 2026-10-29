//! Query proofs grouped by their required dependency surface.
#![recursion_limit = "512"]

#[path = "worth_query_court_query/cost/mod.rs"]
mod cost;
#[path = "worth_query_court_query/determinism/mod.rs"]
mod determinism;
#[path = "worth_query_court_query/query/mod.rs"]
mod query;
