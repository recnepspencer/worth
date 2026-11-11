//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalOwnerMetadata<(), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/owner_metadata.rs",
            "SignalOwnerMetadata",
            128 * 1024,
        );
        owner.field::<u64>("runtime_instance_id");
        owner.field::<SignalOwnerLifecycleIdentity>("lifecycle_identity");
        owner.field::<AtomicUsize>("pending_snapshot_reservations");
        owner.field::<Mutex<SignalOwnerMetadataState<(), (), ()>>>("state");
    }
}
