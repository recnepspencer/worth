//! Relational branch roots, retained basis, and projection views.
#![forbid(unsafe_code)]
#![deny(unreachable_patterns)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]

#[doc(hidden)]
#[path = "internal_port.rs"]
pub mod internal_port;

use internal_port::*;

#[cfg(test)]
#[global_allocator]
static TEST_ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> =
    &stats_alloc::INSTRUMENTED_SYSTEM;

#[cfg(test)]
use worth_relational_contracts::internal_port::inline_type_sizes;
#[cfg(test)]
mod inline_type_sizes_tests;
