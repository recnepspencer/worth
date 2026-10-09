use super::*;

impl RebuiltPhysicalMutationIdempotency {
    pub(in crate::physical_runtime) fn reconcile_verified_pending_release(
        &self,
        pending: &worth_store_recovery_physics::VerifiedPendingWalReleaseCustody,
    ) -> Result<(), PhysicalIdempotencyReopenFailure> {
        self.owner
            .reconcile_verified_pending_release(pending)
            .map_err(|()| PhysicalIdempotencyReopenFailure::PendingReleaseBindingMismatch)
    }

    pub(in crate::physical_runtime) fn into_owner(
        self,
    ) -> Arc<PhysicalMutationIdempotencyRuntimeOwner> {
        self.owner
    }

    pub(in crate::physical_runtime) const fn checkpoint_counters(
        &self,
    ) -> PhysicalBindingCompactionReopenCounters {
        self.checkpoint
    }

    pub(in crate::physical_runtime) const fn wal_members_read(&self) -> u64 {
        self.wal_members_read
    }
}
