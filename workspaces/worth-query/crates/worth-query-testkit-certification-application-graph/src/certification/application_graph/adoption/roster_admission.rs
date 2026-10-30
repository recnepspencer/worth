//! Which rosters a two-rule host will admit at all.
//!
//! A host installs rule contracts; a roster declares which program reads which
//! of them as law. Installation refuses a roster that leaves an installed rule
//! unclaimed, and refuses a program that claims a rule this host never
//! installed. Both refusals happen through the ordinary public install entry.

pub use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
pub use worth_query_host::facade::declaration::application_program::ApplicationProgramDefinition;
pub use worth_query_host::facade::domain::WorthQueryProgramSupportDenial;

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::{
    publish_first_program_alone, publish_with_foreign_rule_rostered,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::ForeignRuleRetentionProgram;
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema;
