//! Query proofs grouped by their product frontier.
#![recursion_limit = "512"]

#[path = "worth_query_court_decl_host/authored_ownership/mod.rs"]
mod authored_ownership;

#[path = "worth_query_court_decl_host/certification/mod.rs"]
mod certification;

#[path = "worth_query_court_decl_host/aspect_patterns/mod.rs"]
mod aspect_patterns;

#[path = "worth_query_court_decl_host/cost/mod.rs"]
mod cost;

#[path = "worth_query_court_decl_host/determinism/mod.rs"]
mod determinism;

#[path = "worth_query_court_decl_host/effect_order/mod.rs"]
mod effect_order;
