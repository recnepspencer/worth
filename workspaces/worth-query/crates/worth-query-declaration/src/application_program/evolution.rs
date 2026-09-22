//! Content identity of authored program meaning as it evolves.

mod revision;
mod semantic_description;
mod semantic_diff;

#[cfg(test)]
mod revision_tests;

pub use revision::ApplicationProgramRevision;
pub(in crate::application_program) use revision::ApplicationProgramRevisionBudgetDenial;
pub use semantic_description::{
    ApplicationSemanticDescription, ApplicationSemanticFact, ApplicationSemanticFamily,
};
pub use semantic_diff::{
    ApplicationProgramMigrationAssessmentRequirement, ApplicationSemanticChange,
    ApplicationSemanticChangeKind, ApplicationSemanticDiff, ApplicationSemanticDiffDenial,
};
