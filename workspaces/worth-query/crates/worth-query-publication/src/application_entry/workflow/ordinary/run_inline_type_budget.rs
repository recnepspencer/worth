//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryOrdinaryWorkflowRunStop>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/workflow/ordinary/run.rs", "WorthQueryOrdinaryWorkflowRunStop", 128 * 1024);
    }
}
