//! Public branch-adoption journeys across the host facade.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
pub use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::{
    publish_on_first_program, SEED_RETENTION,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::presented_request::set_retention;
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::RetentionProgramP1;
pub use worth_query_testkit_document_retention::facade::document_retention_model::readback::read_retention;
pub use worth_query_testkit_document_retention::facade::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

pub const P0_ONLY_RETENTION: u64 = 3;
pub const P1_ONLY_RETENTION: u64 = 15;
