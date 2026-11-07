//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPrimaryGraphSourceAdapter<()>>(
            "workspaces/worth-query/crates/worth-query/src/runtime/primary_graph_source.rs",
            "WorthQueryPrimaryGraphSourceAdapter",
            128 * 1024,
        );
        owner.field::<worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation>("installation");
        owner.field::<worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle>("graph");
        owner.field::<()>("projection");
    }
}
