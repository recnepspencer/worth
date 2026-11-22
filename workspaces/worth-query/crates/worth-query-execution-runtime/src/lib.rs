#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Installed Query execution runtime substrate: product world, provider
//! sessions, managed runs, convergence epochs, artifacts, bindings,
//! advancement phases, and obligation history.

#![forbid(unsafe_code)]
#![recursion_limit = "256"]

mod advancement_phase_internal_port;
mod basis;
mod commit_touches;
mod domain_computation;
mod execution_runtime_internal_port;
mod managed_run_custody_port;
mod ports;
mod product_world_internal_port;

pub(crate) use worth_query_execution_foundation::facade::{
    equality_type_id, execution_digest, relational_snapshot_release,
};

pub mod facade;
