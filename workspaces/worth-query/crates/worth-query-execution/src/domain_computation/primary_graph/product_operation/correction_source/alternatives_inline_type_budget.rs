//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQuerySelectedCorrectionAlternative<'static, ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/correction_source/alternatives.rs", "WorthQuerySelectedCorrectionAlternative", 128 * 1024);
        owner.field::<WorthQuerySelectedCorrectionSource<'static, ()>>("source");
        owner.field::<WorthQueryRecordedCorrectionSource>("root");
        owner.field::<WorthQueryRecordedCorrectionSource>("selected");
        owner.field::<Option<WorthQueryRecordedCorrectionSource>>("active");
        owner.field::<ProductBranchAncestorSelection>("_selected_ancestry");
        owner.field::<Option<ProductBranchAncestorSelection>>("_active_ancestry");
    }
}
