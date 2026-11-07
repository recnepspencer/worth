//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQuerySelectedProductOperation<'static, ()>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/context.rs", "WorthQuerySelectedProductOperation", 128 * 1024);
        owner.field::<&'static WorthQueryPrimaryGraphApplicationRuntime<()>>("application");
        owner.field::<SelectedQueryOrigin>("selection_origin");
        owner.field::<WorthQueryProductBranchLease>("product");
        owner.field::<super::super::super::application_query::resource_lifecycle::WorthQueryApplicationBasisLease>("application_basis");
        owner.field::<Result<
            super::super::super::WorthQuerySelectedProgramInspection,
            super::super::super::WorthQuerySelectedProgramInspectionDenial,
        >>("selected_program");
        owner.field::<Option<
            std::sync::Arc<
                super::super::super::application_output_demand::OriginalSettledProducerObservation,
            >,
        >>("producer");
    }
}
