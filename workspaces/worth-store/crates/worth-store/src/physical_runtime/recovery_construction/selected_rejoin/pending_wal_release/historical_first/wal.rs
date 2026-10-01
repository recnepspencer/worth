//! Exact C.9 member and fate for the earlier, already-published V3 edge.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_canonical_redo_v3, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits,
};
use worth_store_recovery_physics::{RecoveryOperationFate, VerifiedHistoricalPendingWalBatch};

use super::super::super::{wal_fate, SelectedMediaRejoinDenial as Denial};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
    StoreRecoveryOperationFate,
};

pub(super) fn match_batch(
    batch: &VerifiedHistoricalPendingWalBatch,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    cutoff: u64,
    max_decode_scratch: u64,
    format: PhysicalRecordFormatDeclaration,
) -> Result<PersistedPhysicalRecoveryProjection, Denial> {
    let request = batch.descriptor().custody().request();
    let witness = batch.wal_fate();
    if witness.lsn_start() < cutoff || request.idempotency() != batch.chain().descriptor_operation()
    {
        return Err(Denial::WalFate);
    }
    let mut members = sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == request.idempotency());
    let member = members.next().ok_or(Denial::WalFate)?;
    let group = batch.member_group();
    let redo_digest: [u8; 32] = Sha256::digest(member.canonical_redo()).into();
    if members.next().is_some()
        || member.lsn_range().start().get() != witness.lsn_start()
        || member.lsn_range().end_exclusive().get() != witness.lsn_end_exclusive()
        || member.group_identity() != group.group_identity()
        || member.group_member_identity() != group.member_identity()
        || member.group_member_ordinal() != group.member_ordinal()
        || member.group_member_count() != group.member_count()
        || member.group_membership_digest() != group.membership_digest()
        || redo_digest != batch.member_redo_digest()
        || redo_digest != batch.chain().first_redo_sha256()
    {
        return Err(Denial::WalFate);
    }
    let mut overlapping = frames.iter().filter(|frame| {
        frame.lsn_start() < witness.lsn_end_exclusive() && witness.lsn_start() < frame.lsn_end()
    });
    let frame = overlapping.next().ok_or(Denial::WalFate)?;
    if overlapping.next().is_some()
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
    // The sampled C.9 fate remains Indeterminate for an interrupted first
    // process. C.8 may separately reconcile its already-published effect as
    // DurableUnacknowledged; only the checked root delta below licenses that
    // consequence, not a mutation of this raw WAL observation.
    let fate = match batch.chain().first_fate() {
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
        || operation.fate() != fate
        || !matches!(
            batch.operation_fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
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
    if bound.checked_mul(4).ok_or(Denial::BoundExceeded)? > max_decode_scratch {
        return Err(Denial::BoundExceeded);
    }
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: bound,
        record_identities: bound,
        placements: bound,
        segment_updates: bound,
        manifests: bound,
        total_entries: bound.saturating_mul(3),
        inline_allocations: bound,
    };
    let (_, projection) = decode_canonical_redo_v3(
        member.canonical_redo(),
        witness.lsn_start(),
        witness.lsn_end_exclusive(),
        bound,
        None,
        limits,
        format,
    )
    .map_err(|_| Denial::WalFate)?;
    let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } = projection.operation()
    else {
        return Err(Denial::WalFate);
    };
    if projection.source_root_generation() != batch.chain().source_root_generation()
        || binding.record() != batch.descriptor_record()
        || binding.record_payload_sha256() != batch.descriptor_frame_sha256()
        || binding.candidate_root_generation() != batch.chain().first_result_generation()
    {
        return Err(Denial::WalFate);
    }
    Ok(projection)
}
