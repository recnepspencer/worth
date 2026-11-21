#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Installed Query execution primary-graph query evaluation, managed indexes,
//! derived views, handlers, invariants, resources, bootstrap, installation,
//! conditional operations, causal inspection, correction sources, document
//! history, and producer computation.

#![forbid(unsafe_code)]
#![recursion_limit = "256"]

pub(crate) mod domain_computation;
pub(crate) mod ports;
pub(crate) mod ports_internal;

pub(crate) mod publication_boundary {

    pub mod discovery_currentness {
        pub use crate::domain_computation::primary_graph::application_query::{
            WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
        };
    }
}

pub(crate) use worth_query_execution_foundation::facade::{
    equality_type_id, relational_snapshot_release,
};
pub(crate) use worth_query_execution_runtime::facade::basis;

pub mod facade;

#[cfg(test)]
pub(crate) mod inline_type_sizes;
