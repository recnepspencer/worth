//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryWorkflowOperationBindingDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/workflow/operation.rs", "WorthQueryWorkflowOperationBindingDenial", 128 * 1024);
    }
    super::owner::inline_type_budget::collect(census);
}
