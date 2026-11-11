//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalMutationAttempt>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/mutation/handle.rs", "PhysicalMutationAttempt", 128 * 1024);
        owner.field::<PhysicalMutationIdentity>("identity");
        owner.field::<PhysicalMutationIdempotencyKeyIdentity>("idempotency");
        owner.field::<PhysicalMutationRequestFingerprint>("fingerprint");
        owner.field::<PhysicalMutationDeadline>("deadline");
        owner.field::<Weak<crate::physical_runtime::durability::lifecycle::PhysicalMutationRuntimeOwner>>("owner");
        owner.field::<Mutex<()>>("effect_cutover");
        owner.field::<Mutex<PhysicalMutationAttemptState>>("state");
        owner.field::<Condvar>("changed");
        owner.field::<Mutex<Option<Arc<worth_execution::ExecutionInterruptionView>>>>(
            "interruption",
        );
        owner.field::<Mutex<Option<anchored_step::ManagedPublicationCustody>>>("anchored_step");
    }
}
