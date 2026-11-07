//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryHistoryStep>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/step.rs", "WorthQueryHistoryStep", 128 * 1024);
        owner.field::<WorthQueryHistoryNodeId>("id");
        owner.field::<Option<WorthQueryHistoryNodeId>>("parent");
        owner.field::<Option<Arc<str>>>("label");
        owner.field::<WorthQueryProductBranchReadIdentity>("endpoint");
    }
}
