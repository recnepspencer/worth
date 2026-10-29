//! A typed mutation candidate repairs incompatible state inside adoption's
//! single Relational publication rather than becoming a separate operation.

#[path = "migration/unowned_binding.rs"]
pub mod unowned_binding;

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramMigrationPreparationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

pub use crate::facade::certification::document_retention_model::host::publish_on_first_program;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::programs::RetentionProgramP1;
pub use crate::facade::certification::document_retention_model::readback::read_retention;
pub use crate::facade::certification::document_retention_model::retention_entry::{
    SetRetentionIntent, DOCUMENT_IDENTITY,
};
pub use crate::facade::certification::document_retention_model::schema::SetRetentionInput;
pub use crate::facade::certification::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

pub const P0_ONLY_RETENTION: u64 = 3;
pub const P1_VALID_RETENTION: u64 = 15;
