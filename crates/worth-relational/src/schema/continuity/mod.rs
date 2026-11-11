mod authority_input;
mod bundle_admission;
mod errors;
mod transition_canonical_fingerprint;
mod transition_classification;
mod transition_lowering;
mod transition_validation;

pub use bundle_admission::{validate_schema_continuity_bundle, ValidatedSchemaContinuityBundle};
pub use errors::SchemaContinuityBundleIssue;
pub use transition_lowering::lower_schema_transition;
pub use transition_validation::validate_schema_transition;

pub(crate) use authority_input::SchemaContinuityAuthorityInput;
pub(crate) use transition_classification::classify_schema_transition;

pub(super) use transition_canonical_fingerprint::{
    fingerprint_transition, strongest_boundary_visibility, strongest_historical_interpretation,
};
pub(super) use transition_classification::{is_contract_upgrade_policy, is_narrowing};

pub(crate) use bundle_admission::validate_schema_continuity_body;
