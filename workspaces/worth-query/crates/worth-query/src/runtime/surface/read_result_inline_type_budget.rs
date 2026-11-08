//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryReadResult>(
            "workspaces/worth-query/crates/worth-query/src/runtime/surface/read_result.rs",
            "WorthQueryReadResult",
            128 * 1024,
        );
        owner.field::<Vec<WorthQueryEntity>>("rows");
        owner.field::<Option<Vec<WorthQueryEntity>>>("maintenance_source_rows");
        owner.field::<WorthQueryReadReceipt>("receipt");
    }
}
