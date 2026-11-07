//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationRetainedMutationOutcome<(), ()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/mutation/retained.rs", "WorthQueryApplicationRetainedMutationOutcome", 128 * 1024);
        owner.field::<WorthQueryApplicationCommitReceipt>("Committed.receipt");
        owner.field::<()>("Committed.result");
        owner.field::<WorthQueryApplicationReadObservation>("Committed.retained");
    }
}
