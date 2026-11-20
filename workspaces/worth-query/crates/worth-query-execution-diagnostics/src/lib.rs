#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Canonical diagnostic evidence identity and causal inspection protocol for
//! the internal Query execution stack.

#![forbid(unsafe_code)]

mod causal_inspection;
mod evidence_identity;

pub mod facade;
