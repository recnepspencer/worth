//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let _owner = census.owner::<WorthQueryOperationAuthorizationDenialKind>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/denial.rs", "WorthQueryOperationAuthorizationDenialKind", 128 * 1024);
    }
    {
        let mut owner = census.owner::<WorthQueryOperationAuthorizationRefusalKind>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/denial.rs", "WorthQueryOperationAuthorizationRefusalKind", 128 * 1024);
        owner.field::<usize>("ActiveSnapshotCapacityExhausted.maximum_active_snapshots");
    }
}
