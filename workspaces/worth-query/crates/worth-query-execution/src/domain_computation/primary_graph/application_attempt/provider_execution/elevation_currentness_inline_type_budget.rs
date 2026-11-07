//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryElevationCommitCurrentness>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution/elevation_currentness.rs", "WorthQueryElevationCommitCurrentness", 128 * 1024);
        owner.field::<ApplicationCapabilityValidityTimeline>("Window.timeline");
        owner.field::<AspectValue>("Window.issued_at");
        owner.field::<AspectValue>("Window.expires_at");
        owner.field::<ApplicationCapabilityValidityTimeline>("Close.timeline");
        owner.field::<AspectValue>("Close.expires_at");
        owner.field::<WorthQueryElevationClosureKind>("Close.closure_kind");
    }
}
