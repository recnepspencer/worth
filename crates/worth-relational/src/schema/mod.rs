pub(crate) mod aggregate_identity;
pub(crate) mod aspect_plan_source;
pub(crate) mod aspect_plans;
pub(crate) mod continuity;
pub(crate) mod current_publication_ownership;
pub mod data;
mod registry_custody;
pub(crate) mod source_catalog;
pub(crate) use registry_custody::SchemaRegistryCustody;

pub(crate) use aspect_plans::{
    lower_aspect_plans, lower_aspect_plans_for_compilation,
    lower_relation_integrity_plans_for_compilation,
};
#[cfg(test)]
pub(crate) use continuity::classify_schema_transition;
pub(crate) use continuity::SchemaContinuityAuthorityInput;
pub(crate) use continuity::{
    lower_schema_transition, validate_schema_continuity_bundle, validate_schema_transition,
    SchemaContinuityBundleIssue, ValidatedSchemaContinuityBundle,
};

pub(crate) use continuity::validate_schema_continuity_body;

pub(crate) mod compilation_control;
pub(crate) use compilation_control::SchemaCompilationControl;
mod snapshot_compilation;

pub(crate) mod portable_patch;

#[path = "../runtime/initial_schema_installation/recovered_codec_transition/physical_field_basis.rs"]
pub(crate) mod codec_field_comparison;
