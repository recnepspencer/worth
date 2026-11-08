//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQuerySuspendedGeneratedOutput>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/generated_materialization.rs", "WorthQuerySuspendedGeneratedOutput", 128 * 1024);
        owner.field::<WorthQueryProductPublicationBinding>("publication");
        owner.field::<crate::basis::WorthQueryProductBranch>("branch");
        owner.field::<RelationalMaterializationCustody>("custody");
        owner.field::<Arc<WorthQueryApplicationOutputCorrespondence>>("correspondence");
        owner.field::<ProducerQualification>("producer");
    }
    {
        let _owner = census.owner::<WorthQueryGeneratedOutputSuspensionDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/generated_materialization.rs", "WorthQueryGeneratedOutputSuspensionDenial", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQueryGeneratedOutputSuspensionFailure>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/generated_materialization.rs", "WorthQueryGeneratedOutputSuspensionFailure", 128 * 1024);
    }
    super::suspension::inline_type_budget::collect(census);
    super::suspend::inline_type_budget::collect(census);
}
