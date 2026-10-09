//! Immutable observations validated before native sampling storage is admitted.

use super::super::{
    failure::{empty_failure, sample_failure_from_evidence},
    StoreRecoveryBindingSampleDenial as Denial, StoreRecoveryBindingSampleFailure,
    StoreRecoveryCheckpointBindingBasis, StoreRecoveryOperationEvidence,
};
use super::allocation::StoreRecoveryBindingSampleAllocationDenial;
use super::storage::SamplingStorage;
use crate::physical_runtime::durability::PhysicalBindingDecodingContext;
use crate::physical_runtime::{
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalRecoveryCoordination,
};
use std::num::NonZeroU64;
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::VerifiedCheckpointFacts;

pub(super) struct SamplingSource<'coordination> {
    pub(super) store: StableStoreIdentity,
    pub(super) generation: u64,
    pub(super) cutoff: u64,
    pub(super) context: PhysicalBindingDecodingContext,
    pub(super) sealed_basis_digest: [u8; 32],
    pub(super) policy_identity: [u8; 32],
    pub(super) basis: &'coordination StoreRecoveryCheckpointBindingBasis,
    pub(super) evidence: &'coordination [StoreRecoveryOperationEvidence],
}

pub(super) fn validate_source<'coordination>(
    coordination: &'coordination PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    checkpoint: &VerifiedCheckpointFacts,
    maximum_operations: u64,
) -> Result<SamplingSource<'coordination>, StoreRecoveryBindingSampleFailure> {
    let freshness = coordination.freshness();
    freshness.record_binding_sample();
    if !freshness.matches_media_generation(media.media_generation()) {
        return Err(empty_failure(Denial::FreshnessMediaMismatch));
    }
    let source = checkpoint.source();
    let store = media.store_identity();
    if source.identity().store_identity() != store {
        return Err(empty_failure(Denial::ForeignCheckpoint));
    }
    let security = source
        .security_binding()
        .ok_or_else(|| empty_failure(Denial::MissingCheckpointSecurityBinding))?;
    let retention = NonZeroU64::new(security.idempotency_retention_generations())
        .ok_or_else(|| empty_failure(Denial::InvalidCheckpointSecurityBinding))?;
    let context = PhysicalBindingDecodingContext::new(
        store,
        PhysicalDurabilityPolicyIdentity::from_recovery_binding(security.policy_identity()),
        PhysicalIdempotencyPolicy::from_recovery_binding(retention),
    );
    let basis = coordination
        .checkpoint_binding_basis()
        .ok_or_else(|| empty_failure(Denial::InvalidCheckpointBinding))?;
    let evidence = basis.evidence(checkpoint, maximum_operations)?;
    Ok(SamplingSource {
        store,
        generation: checkpoint.compaction_cutover().product_generation(),
        cutoff: checkpoint.compaction_cutover().wal_cutoff_lsn_exclusive(),
        context,
        sealed_basis_digest: security.digest(),
        policy_identity: security.policy_identity(),
        basis,
        evidence,
    })
}

impl SamplingSource<'_> {
    pub(super) fn allocation_failure(
        &self,
        cause: StoreRecoveryBindingSampleAllocationDenial,
    ) -> StoreRecoveryBindingSampleFailure {
        sample_failure_from_evidence(Denial::RecoveryMemoryLimit, self.evidence.iter(), 0, 0)
            .with_allocation_denial(cause)
    }

    pub(super) fn copy_checkpoint_evidence(
        &self,
        storage: &mut SamplingStorage,
    ) -> Result<(), StoreRecoveryBindingSampleFailure> {
        for item in self.evidence {
            storage
                .operations
                .merge(item.clone())
                .map_err(|denial| storage.failure(denial))?;
        }
        Ok(())
    }
}
