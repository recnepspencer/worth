//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalWalAppendPort>(
            "workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port.rs",
            "PhysicalWalAppendPort",
            128 * 1024,
        );
        owner.field::<Weak<PhysicalStoreWorkRuntime>>("runtime");
        owner.field::<PhysicalWorkExecution>("execution");
        owner.field::<PhysicalWorkAdmissionAuthority>("physical");
        owner.field::<PhysicalSchedulerAdmissionOwner>("scheduler");
        owner.field::<Arc<RecordWorkAdmission>>("record");
        owner.field::<PhysicalWalRuntimeOwner>("owner");
        owner.field::<PhysicalDurabilityGroupingRuntimeAuthority>("grouping");
        owner.field::<PhysicalMutationIdempotencyRuntimeAuthority>("idempotency");
        owner.field::<PhysicalDurabilityObservation>("durability");
        owner.field::<Arc<
            OnceLock<
                Arc<crate::physical_runtime::durability::retention::PhysicalPublicationAdmission>,
            >,
        >>("publication");
        #[cfg(feature = "certification-test-authority")]
        owner.field::<Arc<AtomicBool>>("fail_next_member_before_effect");
        #[cfg(feature = "certification-test-authority")]
        owner.field::<Arc<AtomicBool>>("owe_before_maintenance_barrier");
    }
    super::group::inline_type_budget::collect(census);
}
