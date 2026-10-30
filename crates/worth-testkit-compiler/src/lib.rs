//! Compiler-boundary courts consume sessions completed by their outer runner.
//! This crate never launches a compiler. Snapshot checks retain complete rendered
//! diagnostics, source identities, and every coordinate and detail asserted by
//! the existing snapshots. External renderer masks follow those snapshots.
//! Semantic checks bind a declared code and API symbols to one attributed error.

mod denial;
mod diagnostic_layout;
mod diagnostic_lists;
mod diagnostics;
mod paths;
mod refusal;
mod semantic_denial;
mod session;
mod snapshot;
mod source_locations;

#[cfg(test)]
mod fixture_path_tests;

#[cfg(test)]
mod normalization_tests;

#[cfg(test)]
mod relocated_snapshot_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod codegen_tests;

#[cfg(test)]
mod semantic_denial_tests;

pub use denial::{denied, production_denial};
pub use refusal::CompilerFixtureRefusal;
pub use semantic_denial::certification_semantic_denial;
