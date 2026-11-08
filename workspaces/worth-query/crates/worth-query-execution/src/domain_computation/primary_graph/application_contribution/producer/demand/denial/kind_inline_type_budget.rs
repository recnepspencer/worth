//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryOutputDemandDenialKind>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/denial/kind.rs", "WorthQueryOutputDemandDenialKind", 128 * 1024);
    }
}
