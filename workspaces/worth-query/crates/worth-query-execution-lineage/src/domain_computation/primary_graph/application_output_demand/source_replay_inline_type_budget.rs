//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryOutputSourceReplay<'static>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_output_demand/source_replay.rs", "WorthQueryOutputSourceReplay", 128 * 1024);
    }
}
