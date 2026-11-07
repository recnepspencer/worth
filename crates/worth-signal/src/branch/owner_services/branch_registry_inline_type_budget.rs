//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchRegistry<()>>(
            "crates/worth-signal/src/branch/owner_services/branch_registry.rs",
            "SignalBranchRegistry",
            128 * 1024,
        );
        owner.field::<u64>("owner_runtime_instance_id");
        owner.field::<SignalOwnerLifecycleIdentity>("owner_lifecycle_identity");
        owner.field::<usize>("maximum_live_branches");
        owner.field::<usize>("maximum_reservations");
        owner.field::<Mutex<SignalBranchRegistryState<()>>>("state");
        owner.field::<Arc<SignalOwnerServiceCounters>>("counters");
        owner.field::<AtomicBool>("recovered_poison");
    }
}
