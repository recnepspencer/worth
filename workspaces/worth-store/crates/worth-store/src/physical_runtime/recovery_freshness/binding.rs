use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::WalLsnRange;

use crate::physical_runtime::durability::{
    DecodedPhysicalMutationBindingRecord, PersistedPhysicalMutationAttemptBinding,
    PersistedPhysicalMutationFate, PhysicalBindingDecodingContext,
};
use crate::physical_runtime::{
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint,
};

mod accessors;
mod checkpoint_basis;
mod failure;
#[cfg(test)]
mod merge_tests;
mod wal_frame_input;
mod wal_payload;

pub use checkpoint_basis::{
    StoreRecoveryCheckpointBindingBasis, StoreRecoveryCheckpointBindingRebuilder,
};
pub use failure::StoreRecoveryBindingSampleFailure;
use failure::{empty_failure, sample_failure};
pub(super) use wal_frame_input::sample_binding;
use wal_frame_input::RecoveryWalFrameInput;
use wal_payload::decode_wal_member_payload;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecoveryBindingFreshnessSample {
    store: StableStoreIdentity,
    selected_checkpoint_generation: u64,
    sealed_basis_identity: [u8; 32],
    policy_identity: [u8; 32],
    operations: Box<[StoreRecoveryOperationEvidence]>,
    wal_members: Box<[StoreRecoveryWalMember]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecoveryOperationEvidence {
    idempotency_identity: [u8; 32],
    mutation: PhysicalMutationIdentity,
    request_fingerprint: PhysicalMutationRequestFingerprint,
    lease_issuance_generation: u64,
    lease_expiry_generation: u64,
    freshness: StoreRecoveryBindingFreshness,
    fate: StoreRecoveryOperationFate,
    attempt_binding_identity: Option<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecoveryWalMember {
    lsn_range: WalLsnRange,
    operation_identity: [u8; 32],
    group_identity: [u8; 32],
    group_member_identity: [u8; 32],
    group_member_ordinal: u32,
    group_member_count: u32,
    group_membership_digest: [u8; 32],
    canonical_redo: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryBindingFreshness {
    Retained,
    ExpiredAtSelectedCheckpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryOperationFate {
    AcknowledgedDurable,
    DurableUnacknowledged,
    ProvenNoEffect,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryBindingSampleDenial {
    FreshnessMediaMismatch,
    ForeignCheckpoint,
    MissingCheckpointSecurityBinding,
    InvalidCheckpointSecurityBinding,
    InvalidCheckpointBinding,
    InvalidWalMember,
    ConflictingOperationEvidence,
    OperationBindingLimit,
    RedoByteLimit,
}

fn sample_binding_from_frames<'frame, Frame: RecoveryWalFrameInput + 'frame>(
    freshness: &super::PhysicalRecoveryFreshnessAuthority,
    checkpoint_basis: Option<&StoreRecoveryCheckpointBindingBasis>,
    media: &AdmittedRecoveryFilesystemMedia,
    checkpoint: &VerifiedCheckpointStream,
    wal_frames: impl IntoIterator<Item = &'frame Frame>,
    maximum_operation_bindings: u64,
    maximum_redo_bytes: u64,
) -> Result<StoreRecoveryBindingFreshnessSample, StoreRecoveryBindingSampleFailure> {
    freshness.record_binding_sample();
    if !freshness.matches_media_generation(media.media_generation()) {
        return Err(empty_failure(
            StoreRecoveryBindingSampleDenial::FreshnessMediaMismatch,
        ));
    }
    let store = media.store_identity();
    let source = checkpoint.source();
    if source.identity().store_identity() != store {
        return Err(empty_failure(
            StoreRecoveryBindingSampleDenial::ForeignCheckpoint,
        ));
    }
    let security = source.security_binding().ok_or_else(|| {
        empty_failure(StoreRecoveryBindingSampleDenial::MissingCheckpointSecurityBinding)
    })?;
    let retention =
        NonZeroU64::new(security.idempotency_retention_generations()).ok_or_else(|| {
            empty_failure(StoreRecoveryBindingSampleDenial::InvalidCheckpointSecurityBinding)
        })?;
    let policy =
        PhysicalDurabilityPolicyIdentity::from_recovery_binding(security.policy_identity());
    let idempotency = PhysicalIdempotencyPolicy::from_recovery_binding(retention);
    let context = PhysicalBindingDecodingContext::new(store, policy, idempotency);
    let selected_generation = checkpoint.compaction_cutover().product_generation();
    let mut operations = checkpoint_basis
        .ok_or_else(|| empty_failure(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding))?
        .operations(checkpoint, maximum_operation_bindings)?;
    let mut wal_members = Vec::new();
    let mut wal_group_bindings = Vec::new();
    let mut redo_bytes = 0_u64;
    for frame in wal_frames {
        let (binding_bytes, canonical_redo) = decode_wal_member_payload(frame.recovery_payload())
            .map_err(|denial| {
            sample_failure(denial, &operations, wal_members.len(), redo_bytes)
        })?;
        redo_bytes = redo_bytes
            .checked_add(canonical_redo.len() as u64)
            .ok_or_else(|| {
                sample_failure(
                    StoreRecoveryBindingSampleDenial::RedoByteLimit,
                    &operations,
                    wal_members.len(),
                    u64::MAX,
                )
            })?;
        if redo_bytes > maximum_redo_bytes {
            return Err(sample_failure(
                StoreRecoveryBindingSampleDenial::RedoByteLimit,
                &operations,
                wal_members.len(),
                redo_bytes,
            ));
        }
        let redo_digest: [u8; 32] = Sha256::digest(canonical_redo).into();
        let binding = PersistedPhysicalMutationAttemptBinding::decode_from_wal_member(
            binding_bytes,
            context,
            frame.recovery_lsn_range(),
            redo_digest,
        )
        .map_err(|_| {
            sample_failure(
                StoreRecoveryBindingSampleDenial::InvalidWalMember,
                &operations,
                wal_members.len(),
                redo_bytes,
            )
        })?;
        let evidence = evidence_from_persisted(
            &binding,
            selected_generation,
            StoreRecoveryOperationFate::Indeterminate,
        );
        let operation_identity = evidence.idempotency_identity;
        let group = binding.group();
        merge_evidence(&mut operations, evidence, maximum_operation_bindings)
            .map_err(|denial| sample_failure(denial, &operations, wal_members.len(), redo_bytes))?;
        wal_group_bindings.push((binding.mutation(), group, binding.idempotency_identity()));
        wal_members.push(StoreRecoveryWalMember {
            lsn_range: frame.recovery_lsn_range(),
            operation_identity,
            group_identity: group.group_identity().bytes(),
            group_member_identity: group.member_identity().bytes(),
            group_member_ordinal: group.ordinal().get(),
            group_member_count: group.member_count().get(),
            group_membership_digest: group.membership_digest(),
            canonical_redo: canonical_redo.into(),
        });
    }
    let mut groups = BTreeMap::new();
    for binding in wal_group_bindings {
        groups
            .entry(binding.1.group_identity().bytes())
            .or_insert_with(Vec::new)
            .push(binding);
    }
    for group in groups.values_mut() {
        group.sort_unstable_by_key(|binding| binding.1.ordinal().get());
        let first = group.first().ok_or_else(|| {
            sample_failure(
                StoreRecoveryBindingSampleDenial::InvalidWalMember,
                &operations,
                wal_members.len(),
                redo_bytes,
            )
        })?;
        let count = first.1.member_count().get() as usize;
        let membership = first.1.membership_digest();
        let mut members = BTreeSet::new();
        let mut idempotency = BTreeSet::new();
        if group.len() != count
            || group.iter().enumerate().any(|(index, binding)| {
                binding.1.ordinal().get() as usize != index + 1
                    || binding.1.member_count().get() as usize != count
                    || binding.1.membership_digest() != membership
                    || !members.insert(binding.1.member_identity().bytes())
                    || !idempotency.insert(binding.2.bytes())
            })
            || crate::physical_runtime::durability::reopened_membership_digest(
                &group.iter().map(|binding| binding.0).collect::<Vec<_>>(),
                &group
                    .iter()
                    .map(|binding| binding.1.member_identity())
                    .collect::<Vec<_>>(),
                &group.iter().map(|binding| binding.2).collect::<Vec<_>>(),
            ) != membership
        {
            return Err(sample_failure(
                StoreRecoveryBindingSampleDenial::InvalidWalMember,
                &operations,
                wal_members.len(),
                redo_bytes,
            ));
        }
    }
    Ok(StoreRecoveryBindingFreshnessSample {
        store,
        selected_checkpoint_generation: selected_generation,
        sealed_basis_identity: security.digest(),
        policy_identity: security.policy_identity(),
        operations: operations
            .into_values()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        wal_members: wal_members.into_boxed_slice(),
    })
}

fn checkpoint_evidence(
    record: DecodedPhysicalMutationBindingRecord,
    selected_generation: u64,
) -> StoreRecoveryOperationEvidence {
    match record {
        DecodedPhysicalMutationBindingRecord::RebuiltUnsealed(basis)
        | DecodedPhysicalMutationBindingRecord::RebuiltGroupSealed { basis, .. } => {
            evidence_from_basis(
                basis.key(),
                basis.fingerprint(),
                basis.mutation(),
                selected_generation,
                StoreRecoveryOperationFate::Indeterminate,
                None,
            )
        }
        DecodedPhysicalMutationBindingRecord::WalBound { persisted, .. } => {
            evidence_from_persisted(
                &persisted,
                selected_generation,
                StoreRecoveryOperationFate::Indeterminate,
            )
        }
        DecodedPhysicalMutationBindingRecord::Terminal { basis, fate } => {
            let fate = match fate {
                PersistedPhysicalMutationFate::Completed(_) => {
                    StoreRecoveryOperationFate::AcknowledgedDurable
                }
                PersistedPhysicalMutationFate::ProvenNoEffect(_) => {
                    StoreRecoveryOperationFate::ProvenNoEffect
                }
                PersistedPhysicalMutationFate::Indeterminate(_) => {
                    StoreRecoveryOperationFate::Indeterminate
                }
            };
            evidence_from_basis(
                basis.key(),
                basis.fingerprint(),
                basis.mutation(),
                selected_generation,
                fate,
                None,
            )
        }
    }
}

fn evidence_from_persisted(
    binding: &PersistedPhysicalMutationAttemptBinding,
    selected_generation: u64,
    fate: StoreRecoveryOperationFate,
) -> StoreRecoveryOperationEvidence {
    let attempt_binding_identity = Sha256::digest(binding.bytes()).into();
    evidence_from_basis(
        binding.key(),
        binding.fingerprint(),
        binding.mutation(),
        selected_generation,
        fate,
        Some(attempt_binding_identity),
    )
}

fn evidence_from_basis(
    key: &crate::physical_runtime::PhysicalMutationIdempotencyKey,
    fingerprint: PhysicalMutationRequestFingerprint,
    mutation: PhysicalMutationIdentity,
    selected_generation: u64,
    fate: StoreRecoveryOperationFate,
    attempt_binding_identity: Option<[u8; 32]>,
) -> StoreRecoveryOperationEvidence {
    let lease = key.lease();
    StoreRecoveryOperationEvidence {
        idempotency_identity: key.identity().bytes(),
        mutation,
        request_fingerprint: fingerprint,
        lease_issuance_generation: lease.issuance_generation().get(),
        lease_expiry_generation: lease.expiry_generation().get(),
        freshness: if selected_generation >= lease.expiry_generation().get() {
            StoreRecoveryBindingFreshness::ExpiredAtSelectedCheckpoint
        } else {
            StoreRecoveryBindingFreshness::Retained
        },
        fate,
        attempt_binding_identity,
    }
}

fn merge_evidence(
    operations: &mut BTreeMap<[u8; 32], StoreRecoveryOperationEvidence>,
    evidence: StoreRecoveryOperationEvidence,
    maximum: u64,
) -> Result<(), StoreRecoveryBindingSampleDenial> {
    let key = evidence.idempotency_identity;
    if let Some(existing) = operations.get(&key) {
        if existing.mutation != evidence.mutation
            || existing.request_fingerprint != evidence.request_fingerprint
            || existing.lease_issuance_generation != evidence.lease_issuance_generation
            || existing.lease_expiry_generation != evidence.lease_expiry_generation
            || matches!(
                (existing.attempt_binding_identity, evidence.attempt_binding_identity),
                (Some(left), Some(right)) if left != right
            )
        {
            return Err(StoreRecoveryBindingSampleDenial::ConflictingOperationEvidence);
        }
        if conflicting_terminal_fates(existing.fate, evidence.fate) {
            return Err(StoreRecoveryBindingSampleDenial::ConflictingOperationEvidence);
        }
        if existing.fate == StoreRecoveryOperationFate::Indeterminate
            && evidence.fate != StoreRecoveryOperationFate::Indeterminate
        {
            operations.insert(key, evidence);
        }
        return Ok(());
    }
    if operations.len() as u64 >= maximum {
        return Err(StoreRecoveryBindingSampleDenial::OperationBindingLimit);
    }
    operations.insert(key, evidence);
    Ok(())
}

fn conflicting_terminal_fates(
    existing: StoreRecoveryOperationFate,
    incoming: StoreRecoveryOperationFate,
) -> bool {
    existing != StoreRecoveryOperationFate::Indeterminate
        && incoming != StoreRecoveryOperationFate::Indeterminate
        && existing != incoming
}
