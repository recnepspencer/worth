//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationLiveControls>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_query/live/controls.rs", "WorthQueryApplicationLiveControls", 128 * 1024);
        owner.field::<WorthQueryLiveDeliveryControls>("delivery");
        owner.field::<NonZeroUsize>("maximum_materialized_record_count");
    }
}
