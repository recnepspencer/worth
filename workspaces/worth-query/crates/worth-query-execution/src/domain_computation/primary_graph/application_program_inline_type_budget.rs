//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryPreparedRequiredOutputSource>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_program.rs", "WorthQueryPreparedRequiredOutputSource", 128 * 1024);
        owner.field::<u64>("runtime_authority");
        owner.field::<worth_runtime_world::facade::CompositeCommitIdentity>("source_commit");
        owner.field::<worth_runtime_world::facade::ProductBranchIncarnation>("product_occurrence");
        owner.field::<super::super::application_output_demand::WorthQueryOutputDemandRegistry>(
            "owner",
        );
    }
}
