//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryExecutionRuntimeInstaller>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/execution_runtime/runtime_root.rs", "WorthQueryExecutionRuntimeInstaller", 128 * 1024);
        owner.field::<WorthQueryRuntimeAuthorityIdentity>("authority_identity");
        owner.field::<WorthQueryInstallationRuntimeIdentity>("installation_runtime");
        owner.field::<WorthQueryApplicationQueryResourceProfile>("application_query_resources");
        owner.field::<WorthQueryOutputDemandResourceProfile>("output_demand_resources");
        owner.field::<WorthQueryBranchCoordinationResourceProfile>("branch_coordination_resources");
    }
}
