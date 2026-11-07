//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryOperationCloseDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/close.rs", "WorthQueryOperationCloseDenial", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryClosedOperation>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/close.rs", "WorthQueryClosedOperation", 128 * 1024);
        owner.field::<Option<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanupReceipt,
    >>("managed");
        owner.field::<usize>("retired_component_count");
        owner.field::<Option<WorthQueryApplicationProgramOutputUnavailable>>("output_unavailable");
    }
}
