//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOutputDemandKey>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/registry/demand_key.rs", "WorthQueryOutputDemandKey", 128 * 1024);
        owner.field::<OutputFamilyIdentity>("family");
        owner.field::<SourceEpoch>("source");
        owner.field::<WorthQueryProducerApplicability>("applicability");
        owner.field::<String>("producer");
    }
}
