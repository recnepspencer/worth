mod authority_input;
mod bundle_admission;
pub(crate) use worth_relational_contracts::internal_port::schema::continuity::errors;
pub(crate) use worth_relational_contracts::internal_port::schema::continuity::transition_classification;
pub(crate) use worth_relational_contracts::internal_port::schema::continuity::transition_lowering;
pub(crate) use worth_relational_contracts::internal_port::schema::continuity::transition_validation;

pub use bundle_admission::{validate_schema_continuity_bundle, ValidatedSchemaContinuityBundle};
pub use errors::SchemaContinuityBundleIssue;
pub use transition_lowering::lower_schema_transition;
pub use transition_validation::validate_schema_transition;

pub(crate) use authority_input::SchemaContinuityAuthorityInput;
#[cfg(test)]
pub(crate) use transition_classification::classify_schema_transition;

pub(crate) use bundle_admission::validate_schema_continuity_body;
