//! Compiler-boundary courts consume sessions completed by their outer runner.
//! This crate never launches a compiler. Snapshot checks retain complete rendered
//! error diagnostics and normalize only line endings and source-tree relocation.

mod denial;
mod diagnostics;
mod paths;
mod refusal;
mod session;

#[cfg(test)]
mod tests;

pub use denial::{denied, production_denial};
pub use refusal::CompilerFixtureRefusal;
