//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryAftermathCausalRole>(
            "workspaces/worth-query/crates/worth-query-execution-lineage/src/domain_computation/application_aftermath/causal_fact.rs",
            "WorthQueryAftermathCausalRole",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<WorthQueryRecordedCorrectionSource>(
            "workspaces/worth-query/crates/worth-query-execution-lineage/src/domain_computation/application_aftermath/causal_fact.rs",
            "WorthQueryRecordedCorrectionSource",
            128 * 1024,
        );
        owner.field::<worth_runtime_world::facade::CompositeCommitIdentity>("world");
        owner.field::<RelationalCommitReceipt>("relational");
    }
}
