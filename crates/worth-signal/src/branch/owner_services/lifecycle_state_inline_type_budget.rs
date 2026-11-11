//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalOwnerLifecycleState>(
            "crates/worth-signal/src/branch/owner_services/lifecycle_state.rs",
            "SignalOwnerLifecycleState",
            128 * 1024,
        );
        owner.field::<u64>("owner_runtime_instance_id");
        owner.field::<SignalOwnerLifecycleIdentity>("lifecycle_identity");
        owner.field::<AtomicU64>("phase_and_count");
        owner.field::<Arc<SignalOwnerAdmissionTable>>("admission_table");
        owner.field::<Mutex<()>>("transition_gate");
        owner.field::<Condvar>("drain");
        owner.field::<AtomicBool>("cleanup_claimed");
        owner.field::<std::sync::atomic::AtomicUsize>("cleanup_waiters");
        owner.field::<Arc<SignalOwnerServiceCounters>>("counters");
        owner.field::<AtomicBool>("recovered_poison");
        owner.field::<SignalOwnerOperationControlForTest>("operation_control");
    }
}
