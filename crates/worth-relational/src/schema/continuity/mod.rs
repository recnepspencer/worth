mod authority_input;
mod bundle_admission;
mod errors;
pub(crate) mod transition_canonical_fingerprint;
pub(crate) mod transition_classification;
pub(crate) mod transition_lowering;
pub(crate) mod transition_validation;

pub use bundle_admission::{validate_schema_continuity_bundle, ValidatedSchemaContinuityBundle};
pub use errors::SchemaContinuityBundleIssue;
pub use transition_lowering::lower_schema_transition;
pub use transition_validation::validate_schema_transition;

pub(crate) use authority_input::SchemaContinuityAuthorityInput;
#[cfg(test)]
pub(crate) use transition_classification::classify_schema_transition;

pub(crate) use bundle_admission::validate_schema_continuity_body;
