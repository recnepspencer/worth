//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryAftermathCausalRole>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/application_aftermath/causal_fact.rs", "WorthQueryAftermathCausalRole", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryPendingAftermathCausality>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/application_aftermath/causal_fact.rs", "WorthQueryPendingAftermathCausality", 128 * 1024);
        owner.field::<WorthQueryAftermathCausalRole>("role");
        owner.field::<RelationalCommitReceipt>("parent");
        owner.field::<worth_relational::facade::history::BranchId>("target_branch");
        owner.field::<WorthQueryRecordedCorrectionSource>("source");
        owner.field::<WorthQueryCorrectionLineageBinding>("lineage");
        owner.field::<std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >>("correction");
    }
    {
        let mut owner = census.owner::<WorthQueryRecordedCorrectionSource>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/application_aftermath/causal_fact.rs", "WorthQueryRecordedCorrectionSource", 128 * 1024);
        owner.field::<worth_runtime_world::facade::CompositeCommitIdentity>("world");
        owner.field::<RelationalCommitReceipt>("relational");
    }
}
