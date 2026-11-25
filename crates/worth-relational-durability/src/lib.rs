//! Relational durable media, checkpointing, and reconstruction.
#![forbid(unsafe_code)]
#![deny(unreachable_patterns)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]

#[doc(hidden)]
#[path = "internal_port.rs"]
pub mod internal_port;

use internal_port::*;
