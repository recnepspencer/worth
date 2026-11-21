#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Installed Query execution primary-graph application attempts, provider
//! custody, authorization, workflow semantics, aftermath, and live delivery.

#![forbid(unsafe_code)]
#![recursion_limit = "256"]

pub(crate) mod domain_computation;
pub(crate) mod ports;
mod lineage_ports;

pub(crate) use worth_query_execution_foundation::facade::{
    equality_type_id, execution_digest, relational_snapshot_release,
};
pub(crate) use worth_query_execution_runtime::facade::basis;

pub mod facade;

#[cfg(test)]
pub(crate) mod inline_type_sizes;
