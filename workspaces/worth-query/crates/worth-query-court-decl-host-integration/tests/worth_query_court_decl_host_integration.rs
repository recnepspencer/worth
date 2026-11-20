#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Query proofs grouped by their product frontier.
#![recursion_limit = "512"]

#[path = "worth_query_court_decl_host_integration/certification/mod.rs"]
mod certification;
#[path = "worth_query_court_decl_host_integration/cost/mod.rs"]
mod cost;
#[path = "worth_query_court_decl_host_integration/execution/mod.rs"]
mod execution;
#[path = "worth_query_court_decl_host_integration/host/mod.rs"]
mod host;
#[path = "worth_query_court_decl_host_integration/oracle/mod.rs"]
mod oracle;
#[path = "worth_query_court_decl_host_integration/process/mod.rs"]
mod process;

#[path = "worth_query_court_decl_host_integration/dependent_dispatch/mod.rs"]
mod dependent_dispatch;
