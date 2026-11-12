//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationReadObservation>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/retained_read.rs", "WorthQueryApplicationReadObservation", 128 * 1024);
        owner.field::<Arc<RetainedRead>>("retained");
    }
}
