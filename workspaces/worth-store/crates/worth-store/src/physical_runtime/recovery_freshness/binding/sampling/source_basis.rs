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
    AbsentCheckpointWitness, AdmittedPhysicalDurabilityPolicy,
    ConfiguredPhysicalDurabilityDeclaration, PhysicalDurabilityPolicyIdentity,
    PhysicalIdempotencyPolicy, PhysicalRecoveryCoordination,
};
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_proof::TransitionOutcome;
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::VerifiedCheckpointFacts;
use worth_store_wal::WAL_ORIGIN;

const GENERATION_ZERO_BASIS_DOMAIN: &[u8] = b"worth.store.recovery.generation-zero-basis@1";

/// What a binding sample starts from.
#[derive(Clone, Copy)]
pub enum StoreRecoverySamplingBasis<'checkpoint> {
    /// The selected checkpoint: its covered evidence and security binding.
    Checkpoint(&'checkpoint VerifiedCheckpointFacts),
    /// Recovery observed no checkpoint at all: the empty generation-zero
    /// basis, keyed by the sampled coordination's own absence witness, whose
    /// policy is the configured declaration admitted over this media. Every
    /// WAL binding must carry that policy's identity.
    GenerationZero(
        &'checkpoint AbsentCheckpointWitness,
        ConfiguredPhysicalDurabilityDeclaration,
    ),
}

pub(super) struct SamplingSource<'coordination> {
    pub(super) store: StableStoreIdentity,
    pub(super) generation: u64,
    pub(super) cutoff: u64,
    pub(super) context: PhysicalBindingDecodingContext,
    pub(super) sealed_basis_digest: [u8; 32],
    pub(super) policy_identity: [u8; 32],
    /// `None` only for the generation-zero basis, which owns no evidence.
    pub(super) basis: Option<&'coordination StoreRecoveryCheckpointBindingBasis>,
    pub(super) evidence: &'coordination [StoreRecoveryOperationEvidence],
}

pub(super) fn validate_source<'coordination>(
    coordination: &'coordination PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    basis: StoreRecoverySamplingBasis<'_>,
    maximum_operations: u64,
) -> Result<SamplingSource<'coordination>, StoreRecoveryBindingSampleFailure> {
    let freshness = coordination.freshness();
    freshness.record_binding_sample();
    if !freshness.matches_media_generation(media.media_generation()) {
        return Err(empty_failure(Denial::FreshnessMediaMismatch));
    }
    match basis {
        StoreRecoverySamplingBasis::Checkpoint(checkpoint) => {
            checkpoint_source(coordination, media, checkpoint, maximum_operations)
        }
        StoreRecoverySamplingBasis::GenerationZero(absent, declaration) => {
            generation_zero_source(coordination, media, absent, declaration)
        }
    }
}

fn checkpoint_source<'coordination>(
    coordination: &'coordination PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    checkpoint: &VerifiedCheckpointFacts,
    maximum_operations: u64,
) -> Result<SamplingSource<'coordination>, StoreRecoveryBindingSampleFailure> {
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
        basis: Some(basis),
        evidence,
    })
}

/// Only this coordination's absence witness admits the empty basis; nothing
/// before the canonical WAL origin is covered.
fn generation_zero_source<'coordination>(
    coordination: &'coordination PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    absent: &AbsentCheckpointWitness,
    declaration: ConfiguredPhysicalDurabilityDeclaration,
) -> Result<SamplingSource<'coordination>, StoreRecoveryBindingSampleFailure> {
    if !absent.binds(coordination) {
        return Err(empty_failure(Denial::GenerationZeroWithoutAbsentCheckpoint));
    }
    let policy = generation_zero_policy(media, declaration)
        .ok_or_else(|| empty_failure(Denial::GenerationZeroPolicyUnavailable))?;
    let store = media.store_identity();
    let identity = policy.identity();
    let mut digest = Sha256::new();
    digest.update(GENERATION_ZERO_BASIS_DOMAIN);
    digest.update(identity.bytes());
    Ok(SamplingSource {
        store,
        generation: 0,
        cutoff: WAL_ORIGIN.lsn().get(),
        context: PhysicalBindingDecodingContext::new(
            store,
            identity,
            PhysicalIdempotencyPolicy::from_recovery_binding(
                policy.idempotency_policy().retention().get(),
            ),
        ),
        sealed_basis_digest: digest.finalize().into(),
        policy_identity: identity.bytes(),
        basis: None,
        evidence: &[],
    })
}

fn generation_zero_policy(
    media: &AdmittedRecoveryFilesystemMedia,
    declaration: ConfiguredPhysicalDurabilityDeclaration,
) -> Option<AdmittedPhysicalDurabilityPolicy> {
    let basis = media.physical_durability_admission_basis().ok()?;
    match declaration.admit(basis).into_raw() {
        TransitionOutcome::Success(policy) => Some(policy),
        _ => None,
    }
}

impl SamplingSource<'_> {
    /// Only the generation-zero basis owns no checkpoint binding.
    pub(super) const fn is_generation_zero(&self) -> bool {
        self.basis.is_none()
    }

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
