//! Content identity of authored program meaning as it evolves.

mod revision;

#[cfg(test)]
mod revision_tests;

pub use revision::ApplicationProgramRevision;
pub(in crate::application_program) use revision::ApplicationProgramRevisionBudgetDenial;
