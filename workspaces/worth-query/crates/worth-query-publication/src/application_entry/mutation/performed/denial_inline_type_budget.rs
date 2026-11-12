//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryPerformedMutationExecutionDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/mutation/performed/denial.rs", "WorthQueryPerformedMutationExecutionDenial", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQueryRequiredOutputPreparationDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/mutation/performed/denial.rs", "WorthQueryRequiredOutputPreparationDenial", 128 * 1024);
    }
    super::recovery_posture::inline_type_budget::collect(census);
}
