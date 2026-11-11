//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryHistoryGroupCompletionKind>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/group/results.rs", "WorthQueryHistoryGroupCompletionKind", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryHistoryGroupCompletions<()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/group/results.rs", "WorthQueryHistoryGroupCompletions", 128 * 1024);
        owner.field::<Vec<()>>("items");
        owner.field::<Option<worth_execution::ExecutionMemoryReservation>>("_memory");
    }
}
