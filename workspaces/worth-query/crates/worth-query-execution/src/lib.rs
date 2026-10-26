#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Installed Query execution authority.
//!
//! Admission decides resource eligibility. This package consumes that proof,
//! mints attempt-local provider sessions, and carries execution evidence.

#![forbid(unsafe_code)]
// Auto-trait walks through retained Relational trees exceed the default
// depth in both ownership courts and rustdoc.
#![recursion_limit = "256"]

mod basis;
mod domain_computation;
mod equality_type_id;
mod execution_digest;
mod relational_snapshot_release;

pub mod facade;
pub mod publication_boundary;

#[cfg(test)]
mod test_request_policy;

#[cfg(test)]
#[path = "test_support/bridge_host_request.rs"]
mod bridge_host_request;

#[cfg(test)]
#[path = "test_support/process_deadline.rs"]
mod process_deadline;

#[cfg(test)]
mod test_read_stops;
