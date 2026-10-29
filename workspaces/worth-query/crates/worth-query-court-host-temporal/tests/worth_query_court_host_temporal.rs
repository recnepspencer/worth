//! Query proofs grouped by their required dependency surface.
#![recursion_limit = "512"]

#[path = "worth_query_court_host_temporal/cost/mod.rs"]
mod cost;
#[path = "worth_query_court_host_temporal/host/mod.rs"]
mod host;
#[path = "worth_query_court_host_temporal/oracle/mod.rs"]
mod oracle;
#[path = "worth_query_court_host_temporal/process/mod.rs"]
mod process;
