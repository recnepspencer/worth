//! Relational lineage and causal history indexing.
#![forbid(unsafe_code)]
#![deny(unreachable_patterns)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]

#[doc(hidden)]
#[path = "internal_port.rs"]
pub mod internal_port;

use internal_port::*;

use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port as storage;
