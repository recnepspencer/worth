pub mod authority_input;
pub mod bundle_admission;
pub use worth_relational_contracts::internal_port::schema::continuity::errors;
pub use worth_relational_contracts::internal_port::schema::continuity::transition_classification;
pub use worth_relational_contracts::internal_port::schema::continuity::transition_lowering;
pub use worth_relational_contracts::internal_port::schema::continuity::transition_validation;

pub use bundle_admission::{validate_schema_continuity_bundle, ValidatedSchemaContinuityBundle};
pub use errors::SchemaContinuityBundleIssue;
pub use transition_lowering::lower_schema_transition;
pub use transition_validation::validate_schema_transition;

pub use authority_input::SchemaContinuityAuthorityInput;
pub use transition_classification::classify_schema_transition;

pub use bundle_admission::validate_schema_continuity_body;
