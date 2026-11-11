//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryDocumentHistoryCandidateDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/candidate.rs", "WorthQueryDocumentHistoryCandidateDenial", 128 * 1024);
    }
}
