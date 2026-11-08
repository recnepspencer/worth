//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryHistoryEditDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/edit.rs", "WorthQueryHistoryEditDenial", 128 * 1024);
    }
    {
        let _owner = census.owner::<WorthQueryHistoryEditPosture>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/edit.rs", "WorthQueryHistoryEditPosture", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryHistoryEditAdvance<()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/history/edit.rs", "WorthQueryHistoryEditAdvance", 128 * 1024);
        owner.field::<()>("Refused.edit");
        owner.field::<WorthQueryHistoryEditDenial>("Refused.denial");
    }
}
