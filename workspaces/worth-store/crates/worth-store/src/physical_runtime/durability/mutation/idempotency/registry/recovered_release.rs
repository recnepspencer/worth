//! Exact post-redo terminal binding for a Store-rejoined pending V3 release.
//! The old C.9 fate remains in the checkpoint Batch witness; this updates
//! only the Serving idempotency registry after the selected root was published.

use worth_store_recovery_physics::{RecoveryOperationFate, VerifiedPendingWalReleaseCustody};

use super::{PhysicalMutationIdempotencyBindingState, PhysicalMutationIdempotencyRegistry};
use crate::physical_runtime::{
    durability::mutation::idempotency::{
        binding_compaction::drop_material, fate::PersistedPhysicalMutationFate,
    },
    CompletedPhysicalMutationFact, PhysicalMutationIdempotencyKeyIdentity,
};

impl PhysicalMutationIdempotencyRegistry {
    pub(in crate::physical_runtime) fn reconcile_verified_pending_release(
        &mut self,
        claim: &VerifiedPendingWalReleaseCustody,
    ) -> Result<(), ()> {
        let descriptor = claim.descriptor();
        let base = descriptor.base();
        let request = descriptor.custody().request();
        let published = claim.published_root().ok_or(())?;
        let key =
            PhysicalMutationIdempotencyKeyIdentity::from_observed_bytes(request.idempotency());
        if !self.recovered_complete
            || self.store_identity().bytes() != base.store()
            || published.generation() != base.candidate_root_generation()
            || claim.published_root_sha256().is_none()
            || claim.descriptor_record() == claim.reservation_record()
        {
            return Err(());
        }
        let state = self.bindings.get_mut(&key).ok_or(())?;
        let (basis, persisted) = match state {
            PhysicalMutationIdempotencyBindingState::WalBound { basis, persisted } => {
                (basis.clone(), persisted.clone())
            }
            PhysicalMutationIdempotencyBindingState::Terminal {
                basis,
                fate,
                last_compacted: None,
            } if claim.operation_fate() == RecoveryOperationFate::Indeterminate => (
                basis.clone(),
                fate.indeterminate_wal_binding().ok_or(())?.clone(),
            ),
            _ => return Err(()),
        };
        let lease = basis.key().lease();
        let range = persisted.member().lsn_range();
        let group = persisted.group();
        let claimed_group = claim.member_group();
        if basis.key().caller_material().bytes()
            != drop_material(base.store(), base.reclaim_attempt())
            || basis.key().identity().bytes() != request.idempotency()
            || basis.fingerprint().bytes() != request.fingerprint()
            || basis.mutation().store_identity().bytes() != base.store()
            || lease.store_identity().bytes() != base.store()
            || lease.issuance_generation().get() != request.lease_issuance_generation()
            || lease.expiry_generation().get() != request.lease_expiry_generation()
            || persisted.key() != basis.key()
            || persisted.fingerprint() != basis.fingerprint()
            || persisted.mutation() != basis.mutation()
            || range.start().get() != claim.wal_fate().lsn_start()
            || range.end_exclusive().get() != claim.wal_fate().lsn_end_exclusive()
            || persisted.member().member_identity().bytes() != claimed_group.member_identity()
            || group.group_identity().bytes() != claimed_group.group_identity()
            || group.ordinal().get() != claimed_group.member_ordinal()
            || group.member_count().get() != claimed_group.member_count()
            || group.membership_digest() != claimed_group.membership_digest()
            || persisted.redo_digest() != claim.member_redo_digest()
        {
            return Err(());
        }
        let fact = CompletedPhysicalMutationFact::from_verified_pending_release(
            &persisted,
            claim.descriptor_record(),
            published.generation(),
        );
        *state = PhysicalMutationIdempotencyBindingState::Terminal {
            basis,
            fate: PersistedPhysicalMutationFate::completed(persisted, fact),
            last_compacted: None,
        };
        Ok(())
    }
}
