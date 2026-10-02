//! Exact sampled C.9 member and current canonical projection for one released edge.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_canonical_redo_v3, decode_canonical_redo_v3_with_storage,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_recovery_physics::{
    RecoveryOperationFate, VerifiedOrderedPendingWalReleaseBatch,
    VerifiedOrderedReleasedHeadReplayV14, VerifiedReleasedRootEdge,
};

use super::super::super::{
    resident::{
        decode_storage::{decode_denial, RejoinDecodeStorage, RejoinDecoded},
        StoreRejoinResidentLedger,
    },
    wal_fate, SelectedMediaRejoinDenial as Denial,
};
use super::super::delta;
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, PhysicalRecoveryReadAllocation,
    StoreRecoveryBindingFreshnessSample, StoreRecoveryOperationFate,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn matched_projection(
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    replay: &VerifiedOrderedReleasedHeadReplayV14,
    source_generation: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    retained_peak: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<PersistedPhysicalRecoveryProjection, Denial> {
    matched_projection_inner(
        edge,
        batch,
        replay,
        source_generation,
        sample,
        frames,
        cutoff,
        |bytes, start, end, bound, limits| {
            if bound.checked_mul(4).ok_or(Denial::BoundExceeded)?
                > delta::MAX_TRANSITION_MEMORY
                    .checked_sub(retained_peak)
                    .ok_or(Denial::BoundExceeded)?
            {
                return Err(Denial::BoundExceeded);
            }
            let (_, projection) =
                decode_canonical_redo_v3(bytes, start, end, bound, None, limits, format)
                    .map_err(|_| Denial::WalFate)?;
            Ok(projection)
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn matched_projection_with_storage(
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    replay: &VerifiedOrderedReleasedHeadReplayV14,
    source_generation: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    format: PhysicalRecordFormatDeclaration,
    window: &PhysicalRecoveryReadAllocation<'_>,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<RejoinDecoded<PersistedPhysicalRecoveryProjection>, Denial> {
    let mut storage = RejoinDecodeStorage::new(window, resident);
    let projection = matched_projection_inner(
        edge,
        batch,
        replay,
        source_generation,
        sample,
        frames,
        cutoff,
        |bytes, start, end, bound, limits| {
            let (_, projection) = decode_canonical_redo_v3_with_storage(
                bytes,
                start,
                end,
                bound,
                limits,
                format,
                &mut storage,
            )
            .map_err(decode_denial)?;
            Ok(projection)
        },
    )?;
    Ok(RejoinDecoded::new(projection, storage.into_charge()))
}

#[allow(clippy::too_many_arguments)]
fn matched_projection_inner(
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    replay: &VerifiedOrderedReleasedHeadReplayV14,
    source_generation: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    decode: impl FnOnce(
        &[u8],
        u64,
        u64,
        u64,
        PhysicalRecoveryProjectionDecodeLimits,
    ) -> Result<PersistedPhysicalRecoveryProjection, Denial>,
) -> Result<PersistedPhysicalRecoveryProjection, Denial> {
    let request = batch.descriptor().custody().request();
    let witness = batch.wal_fate();
    let range = edge.lsn();
    let attached = replay.replay();
    if witness.lsn_start() < cutoff
        || witness.lsn_start() != range.start().get()
        || witness.lsn_end_exclusive() != range.end_exclusive().get()
        || request.idempotency() != edge.operation()
        || replay.source_root_frame_sha256() != edge.source_root_frame_sha256()
        || replay.result_root_frame_sha256() != edge.result_root_frame_sha256()
        || attached.lsn_range() != Some(range)
        || attached.operation() != edge.operation()
        || attached.group() != edge.group()
        || attached.fate() != edge.fate()
        || attached.canonical_redo_sha256() != edge.redo_sha256()
    {
        return Err(Denial::WalFate);
    }
    let mut members = sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == edge.operation());
    let member = members.next().ok_or(Denial::WalFate)?;
    let group = edge.group();
    let redo_sha: [u8; 32] = Sha256::digest(member.canonical_redo()).into();
    if members.next().is_some()
        || member.lsn_range() != range
        || member.group_identity() != group.group_identity()
        || member.group_member_identity() != group.member_identity()
        || member.group_member_ordinal() != group.member_ordinal()
        || member.group_member_count() != group.member_count()
        || member.group_membership_digest() != group.membership_digest()
        || redo_sha != edge.redo_sha256()
    {
        return Err(Denial::WalFate);
    }
    // The sample and exact selected frame come from one Store-qualified WAL
    // inventory; the facade compares that inventory with its final reread.
    let mut overlaps = frames.iter().filter(|frame| {
        frame.lsn_start() < witness.lsn_end_exclusive() && witness.lsn_start() < frame.lsn_end()
    });
    let frame = overlaps.next().ok_or(Denial::WalFate)?;
    if overlaps.next().is_some()
        || frame.lsn_start() != witness.lsn_start()
        || frame.lsn_end() != witness.lsn_end_exclusive()
        || frame.identity_digest() != witness.identity_digest()
        || frame.payload_digest() != witness.payload_digest()
    {
        return Err(Denial::WalFate);
    }
    let mut operations = sample
        .operations()
        .iter()
        .filter(|operation| operation.idempotency_identity() == request.idempotency());
    let operation = operations.next().ok_or(Denial::WalFate)?;
    let raw_fate = match edge.fate() {
        RecoveryOperationFate::AcknowledgedDurable => {
            StoreRecoveryOperationFate::AcknowledgedDurable
        }
        RecoveryOperationFate::DurableUnacknowledged => {
            StoreRecoveryOperationFate::DurableUnacknowledged
        }
        RecoveryOperationFate::Indeterminate => StoreRecoveryOperationFate::Indeterminate,
        RecoveryOperationFate::ProvenNoEffect => return Err(Denial::WalFate),
    };
    if operations.next().is_some()
        || operation.fate() != raw_fate
        || !matches!(
            batch.operation_fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
                | RecoveryOperationFate::Indeterminate
        )
        || operation.request_fingerprint().bytes() != request.fingerprint()
        || operation.lease_issuance_generation() != request.lease_issuance_generation()
        || operation.lease_expiry_generation() != request.lease_expiry_generation()
        || wal_fate::expected_drop_key(
            sample.store_identity().bytes(),
            batch.descriptor().base().reclaim_attempt(),
            sample.policy_identity(),
            request.lease_issuance_generation(),
            request.lease_expiry_generation(),
        ) != request.idempotency()
    {
        return Err(Denial::WalFate);
    }
    let bound = member.canonical_redo().len() as u64;
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: bound,
        record_identities: bound,
        placements: bound,
        segment_updates: bound,
        manifests: bound,
        total_entries: bound.saturating_mul(3),
        inline_allocations: bound,
    };
    let projection = decode(
        member.canonical_redo(),
        witness.lsn_start(),
        witness.lsn_end_exclusive(),
        bound,
        limits,
    )?;
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        binding,
        head_effect,
    } = projection.operation()
    else {
        return Err(Denial::WalFate);
    };
    if projection.source_root_generation() != source_generation
        || binding.record() != edge.descriptor_record()
        || binding.record_payload_sha256() != edge.descriptor_frame_sha256()
        || binding.candidate_root_generation()
            != source_generation
                .checked_add(1)
                .ok_or(Denial::BoundExceeded)?
        || head_effect.as_ref() != Some(attached.effect())
    {
        return Err(Denial::WalFate);
    }
    Ok(projection)
}
