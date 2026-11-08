//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<ExpectedSourceQualification>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/generated_materialization/suspend.rs", "ExpectedSourceQualification", 128 * 1024);
        owner.field::<u64>("runtime_authority");
        owner.field::<worth_query_installation::facade::ApplicationSchemaBindingIdentity>("schema");
        owner.field::<crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding>("scope");
        owner.field::<crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity>("runtime_source_identity");
        owner.field::<crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity>("checkpoint_source_identity");
        owner.field::<crate::basis::WorthQueryProductBranchReadIdentity>("selection");
    }
}
