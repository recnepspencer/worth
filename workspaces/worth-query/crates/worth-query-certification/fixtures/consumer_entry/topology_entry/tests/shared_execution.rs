//! External consumer proof of shared execution, classified by cost.
#![recursion_limit = "512"]
#[allow(unused_imports)]
#[path = "worth_query_topology_entry/fast/mod.rs"]
mod fast;
#[allow(dead_code, unused_imports)]
#[path = "worth_query_topology_entry/shared_execution.rs"]
mod shared_execution;
