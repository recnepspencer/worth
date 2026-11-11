//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalConditionalTemporalRegistry>("crates/worth-signal/src/branch/owner_services/conditional_execution/temporal/registry.rs", "SignalConditionalTemporalRegistry", 128 * 1024);
        owner.field::<u64>("owner_runtime_instance_id");
        owner.field::<SignalOwnerLifecycleIdentity>("owner_lifecycle_identity");
        owner.field::<Mutex<SignalConditionalTemporalRegistryState>>("state");
    }
}
