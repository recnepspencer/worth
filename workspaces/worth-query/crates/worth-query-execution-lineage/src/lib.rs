#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Installed Query execution primary-graph lineage, output demand, invariant
//! projection, partitioned computation, handlers, checkpoints, and domain
//! semantics.

#![forbid(unsafe_code)]
#![recursion_limit = "256"]

pub(crate) mod domain_computation;
pub(crate) mod ports;
mod runtime_ports;

pub(crate) use worth_query_execution_foundation::facade::{
    equality_type_id, execution_digest, relational_snapshot_release,
};
pub(crate) use worth_query_execution_runtime::facade::basis;

pub mod facade;

#[cfg(test)]
mod inline_type_sizes;
