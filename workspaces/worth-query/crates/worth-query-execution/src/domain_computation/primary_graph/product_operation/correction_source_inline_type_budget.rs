//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryCorrectionSelectionDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source.rs", "WorthQueryCorrectionSelectionDenial", 128 * 1024);
    }
    super::alternatives::inline_type_budget::collect(census);
    super::disclosure::inline_type_budget::collect(census);
}
