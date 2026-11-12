//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryAcceptedOutputAuthority>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry.rs", "WorthQueryAcceptedOutputAuthority", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryOutputDemandRegistry>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry.rs", "WorthQueryOutputDemandRegistry", 128 * 1024);
        owner.field::<Arc<Mutex<DemandRegistryState>>>("state");
    }
    {
        let mut owner = census.owner::<WorthQueryOutputDemandInterest>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry.rs", "WorthQueryOutputDemandInterest", 128 * 1024);
        owner.field::<WorthQueryOutputDemandKey>("key");
        owner.field::<bool>("requires_output");
        owner.field::<WorthQueryOutputDemandNotifications>("notifications");
        owner.field::<WorthQueryOutputDemandRegistry>("owner");
    }
    super::demand_key::inline_type_budget::collect(census);
    super::checkpoint::inline_type_budget::collect(census);
}
