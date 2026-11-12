//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryAdvancementDenial>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/request_execution/advancement.rs", "WorthQueryAdvancementDenial", 128 * 1024);
    }
    super::phase_custody::inline_type_budget::collect(census);
    super::bootstrap_custody::inline_type_budget::collect(census);
    super::observation::inline_type_budget::collect(census);
}
