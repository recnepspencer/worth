//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryWorkflowAdvancePreparationDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/workflow/progress/preparation_denial.rs", "WorthQueryWorkflowAdvancePreparationDenial", 128 * 1024);
    }
}
