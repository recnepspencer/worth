//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryWorkflowOperationAdvance<(), (), ()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/workflow/mod.rs", "WorthQueryWorkflowOperationAdvance", 128 * 1024);
        owner.field::<()>("Refused.operation");
        owner.field::<WorthQueryWorkflowOperationDenial>("Refused.denial");
    }
    {
        let _owner = census.owner::<WorthQueryWorkflowOperationDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/workflow/mod.rs", "WorthQueryWorkflowOperationDenial", 128 * 1024);
    }
    super::advance::inline_type_budget::collect(census);
}
