//! The foundational_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "certification.rs"]
mod certification;
#[path = "compile_time_boundaries.rs"]
mod compile_time_boundaries;
#[path = "expressions.rs"]
mod expressions;
