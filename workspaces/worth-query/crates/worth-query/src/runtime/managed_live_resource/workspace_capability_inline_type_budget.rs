//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryManagedLiveWorkspaceCapability>("workspaces/worth-query/crates/worth-query/src/runtime/managed_live_resource/workspace_capability.rs", "WorthQueryManagedLiveWorkspaceCapability", 128 * 1024);
        owner.field::<Mutex<Vec<WorthQueryAbandonedManagedLiveResource>>>("abandoned_resources");
        owner.field::<Mutex<
            BTreeMap<
                crate::runtime::WorthQuerySharedExecutionOwnerIdentity,
                Vec<crate::runtime::WorthQuerySharedProjectionLeaseToken>,
            >,
        >>("abandoned_shared_projection_leases");
    }
}
