//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPrimaryGraphBackendHandle>("workspaces/worth-query/crates/worth-query/src/runtime/backend/primary_graph_runtime.rs", "WorthQueryPrimaryGraphBackendHandle", 128 * 1024);
        owner.field::<WorthQueryPrimaryGraphIntegrationHandle>("integration");
    }
}
