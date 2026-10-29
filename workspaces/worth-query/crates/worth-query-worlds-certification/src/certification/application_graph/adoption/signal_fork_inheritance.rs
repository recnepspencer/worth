//! A fork that takes its own Signal component still runs its source's program.
//!
//! Branch program activation is Relational truth carried through World, so
//! which program an occurrence runs must not depend on whether the fork shares
//! its source's exact Signal basis or forks Signal alongside Relational.

pub use crate::facade::certification::document_retention_model::host::{
    publish_on_second_program, SEED_RETENTION,
};
pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::readback::read_retention;
pub use crate::facade::certification::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

/// A value only the second program's law admits.
pub const HIGH_RETENTION: u64 = 15;
/// A value only the first program's law admits.
pub const LOW_RETENTION: u64 = 3;
