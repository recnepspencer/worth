//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryJournalReplayRegistry>(
            "workspaces/worth-query/crates/worth-query/src/runtime/journal_replay/registry.rs",
            "WorthQueryJournalReplayRegistry",
            128 * 1024,
        );
        owner.field::<Vec<WorthQueryJournalReplayEntry>>("entries");
        owner.field::<WorthQueryJournalReplayCounters>("counters");
    }
}
