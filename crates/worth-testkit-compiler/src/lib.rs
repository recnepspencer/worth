//! Compiler-boundary courts consume sessions completed by their outer runner.
//! This crate never launches a compiler. Snapshot checks retain complete rendered
//! diagnostics, source identities, and every coordinate and detail asserted by
//! the existing snapshots. External renderer masks follow those snapshots.

mod denial;
mod diagnostic_layout;
mod diagnostic_lists;
mod diagnostics;
mod paths;
mod refusal;
mod session;
mod snapshot;
mod source_locations;

#[cfg(test)]
mod normalization_tests;

#[cfg(test)]
mod relocated_snapshot_tests;

#[cfg(test)]
mod tests;

pub use denial::{denied, production_denial};
pub use refusal::CompilerFixtureRefusal;
