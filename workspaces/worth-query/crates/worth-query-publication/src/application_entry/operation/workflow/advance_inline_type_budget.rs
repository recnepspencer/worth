//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorkflowStep>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/workflow/advance.rs", "WorkflowStep", 128 * 1024);
    }
}
