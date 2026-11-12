//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryOutputDemandControls>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/demand/request.rs", "WorthQueryOutputDemandControls", 128 * 1024);
        owner.field::<Option<NonZeroUsize>>("maximum_settlement_attempts");
    }
    {
        let _owner = census.owner::<WorthQueryApplicationOutputDemandDenial>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/demand/request.rs", "WorthQueryApplicationOutputDemandDenial", 128 * 1024);
    }
}
