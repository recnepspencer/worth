pub use std::num::NonZeroUsize;

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchSetAdoptionProgress,
    WorthQueryBranchSetAdoptionResumeDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;

pub use super::{fork, target_revision, P0_ONLY_RETENTION, P1_ONLY_RETENTION};
pub use crate::facade::certification::document_retention_model::host::{
    publish_on_first_program, SEED_RETENTION,
};
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

pub fn coverage(
    host: &crate::facade::certification::document_retention_model::host::DocumentRetentionRuntime<
        crate::facade::certification::document_retention_model::programs::RetentionProgramP0,
    >,
    branches: &[worth_query_host::facade::product::WorthQueryProductBranch],
) -> worth_query_host::facade::application_entry::WorthQueryProgramAdoptionCoverage {
    host.runtime()
        .branches()
        .program_adoption_coverage(branches, NonZeroUsize::new(branches.len()).unwrap())
        .unwrap()
}
