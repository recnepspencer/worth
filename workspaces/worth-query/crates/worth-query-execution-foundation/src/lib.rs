#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Foundational execution primitives, identities, outcome vocabularies, and
//! leaf primary-graph types for the internal Query execution stack.

#![forbid(unsafe_code)]

mod domain_computation;
mod equality_type_id;
mod execution_digest;
mod relational_snapshot_release;

pub mod facade;
