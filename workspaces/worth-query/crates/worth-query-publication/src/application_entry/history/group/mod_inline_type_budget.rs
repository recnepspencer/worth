//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryHistoryGroupDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/group/mod.rs", "WorthQueryHistoryGroupDenial", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQueryHistoryGroupPosture>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/group/mod.rs", "WorthQueryHistoryGroupPosture", 128 * 1024);
    }
    super::results::inline_type_budget::collect(census);
}
