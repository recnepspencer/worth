//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryLiveReadResult>(
            "workspaces/worth-query/crates/worth-query/src/runtime/surface/live_read_result.rs",
            "WorthQueryLiveReadResult",
            128 * 1024,
        );
        owner.field::<Vec<crate::memory_workspace::WorthQueryEntity>>("rows");
        owner.field::<Option<Vec<crate::memory_workspace::WorthQueryEntity>>>(
            "maintenance_source_rows",
        );
        owner.field::<WorthQueryLiveReadReceipt>("receipt");
    }
}
