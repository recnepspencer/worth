//! Exact Store-sampled pending C.9 member, fate, and projected placement joins.

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use worth_store_physical_format::{
    decode_canonical_redo_v3, store_namespace::StableStoreIdentity, BlobRecordKind,
    CurrentPhysicalRecordPlacement, PersistedPhysicalRecoveryBlobSemantic,
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits, SelectedRecordContentClass,
};
use worth_store_recovery_physics::{RecoveryOperationFate, VerifiedPendingWalReleaseCustody};

use super::expected_drop_key;
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
    StoreRecoveryOperationFate,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn matches_pending_release(
    store: StableStoreIdentity,
    claim: &VerifiedPendingWalReleaseCustody,
    sample: &StoreRecoveryBindingFreshnessSample,
    selected_wal: &[IntegrityAdmittedRecoveryWalFrame],
    format: PhysicalRecordFormatDeclaration,
) -> bool {
    let descriptor = claim.descriptor();
    let request = descriptor.custody().request();
    let witness = claim.wal_fate();
    let cutoff = claim
        .checkpoint()
        .compaction_cutover()
        .wal_cutoff_lsn_exclusive();
    if sample.store_identity() != store
        || witness.lsn_start() < cutoff
        || request.idempotency() == [0; 32]
    {
        return false;
    }
    let mut members = sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == request.idempotency());
    let Some(member) = members.next() else {
        return false;
    };
    if members.next().is_some()
        || member.lsn_range().start().get() != witness.lsn_start()
        || member.lsn_range().end_exclusive().get() != witness.lsn_end_exclusive()
        || member.group_identity() != claim.member_group().group_identity()
        || member.group_member_identity() != claim.member_group().member_identity()
        || member.group_member_ordinal() != claim.member_group().member_ordinal()
        || member.group_member_count() != claim.member_group().member_count()
        || member.group_membership_digest() != claim.member_group().membership_digest()
        || <[u8; 32]>::from(Sha256::digest(member.canonical_redo())) != claim.member_redo_digest()
    {
        return false;
    }
    let mut frames = selected_wal.iter().filter(|frame| {
        frame.lsn_start() < witness.lsn_end_exclusive() && witness.lsn_start() < frame.lsn_end()
    });
    let Some(frame) = frames.next() else {
        return false;
    };
    if frames.next().is_some()
        || frame.lsn_start() != witness.lsn_start()
        || frame.lsn_end() != witness.lsn_end_exclusive()
        || frame.identity_digest() != witness.identity_digest()
        || frame.payload_digest() != witness.payload_digest()
    {
        return false;
    }
    let mut operations = sample
        .operations()
        .iter()
        .filter(|operation| operation.idempotency_identity() == request.idempotency());
    let Some(operation) = operations.next() else {
        return false;
    };
    let expected_fate = match claim.operation_fate() {
        RecoveryOperationFate::AcknowledgedDurable => {
            StoreRecoveryOperationFate::AcknowledgedDurable
        }
        RecoveryOperationFate::DurableUnacknowledged => {
            StoreRecoveryOperationFate::DurableUnacknowledged
        }
        RecoveryOperationFate::Indeterminate => StoreRecoveryOperationFate::Indeterminate,
        RecoveryOperationFate::ProvenNoEffect => return false,
    };
    if operations.next().is_some()
        || operation.fate() != expected_fate
        || operation.request_fingerprint().bytes() != request.fingerprint()
        || operation.lease_issuance_generation() != request.lease_issuance_generation()
        || operation.lease_expiry_generation() != request.lease_expiry_generation()
        || expected_drop_key(
            store.bytes(),
            descriptor.base().reclaim_attempt(),
            sample.policy_identity(),
            request.lease_issuance_generation(),
            request.lease_expiry_generation(),
        ) != request.idempotency()
    {
        return false;
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
    let Ok((_, projection)) = decode_canonical_redo_v3(
        member.canonical_redo(),
        witness.lsn_start(),
        witness.lsn_end_exclusive(),
        bound,
        None,
        limits,
        format,
    ) else {
        return false;
    };
    let PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding) = projection.blob_semantic()
    else {
        return false;
    };
    binding.record() == claim.descriptor_record()
        && binding.record_payload_sha256() == claim.descriptor_frame_sha256()
        && binding.candidate_root_generation() == descriptor.base().candidate_root_generation()
        && projection.source_root_generation() == claim.source_root().generation()
}

/// Only call after `matches_pending_release` succeeded for this exact C.9
/// sample. Rebind the same unique member digest before yielding its canonical
/// projected placements to Store's independent source-to-result check.
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn matched_pending_projection(
    claim: &VerifiedPendingWalReleaseCustody,
    sample: &StoreRecoveryBindingFreshnessSample,
    format: PhysicalRecordFormatDeclaration,
) -> Option<PersistedPhysicalRecoveryProjection> {
    let mut members = sample.wal_members().iter().filter(|member| {
        member.operation_identity() == claim.descriptor().custody().request().idempotency()
    });
    let member = members.next()?;
    if members.next().is_some()
        || member.group_identity() != claim.member_group().group_identity()
        || member.group_member_identity() != claim.member_group().member_identity()
        || <[u8; 32]>::from(Sha256::digest(member.canonical_redo())) != claim.member_redo_digest()
    {
        return None;
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
    let (_, projection) = decode_canonical_redo_v3(
        member.canonical_redo(),
        claim.wal_fate().lsn_start(),
        claim.wal_fate().lsn_end_exclusive(),
        bound,
        None,
        limits,
        format,
    )
    .ok()?;
    (projection.source_root_generation() == claim.source_root().generation()
        && projection.segment_updates().is_empty()
        && projection.root_state().inline_allocations().is_empty()
        && projection.root_state().last_inline_record().is_none()
        && projection.root_state().last_inline_segment().is_none())
    .then_some(projection)
}

/// Every newly published placement must have one and only one exact C.9
/// member on this pending source, and no such member may introduce an omitted
/// placement. Already-selected identical routes are not new publications.
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn matches_pending_projected_set(
    claim: &VerifiedPendingWalReleaseCustody,
    sample: &StoreRecoveryBindingFreshnessSample,
    source: &[CurrentPhysicalRecordPlacement],
    projected: &[CurrentPhysicalRecordPlacement],
    format: PhysicalRecordFormatDeclaration,
) -> bool {
    let mut witnessed = BTreeSet::new();
    let cutoff = claim
        .checkpoint()
        .compaction_cutover()
        .wal_cutoff_lsn_exclusive();
    let end = claim.wal_fate().lsn_end_exclusive();
    for member in sample.wal_members() {
        let start = member.lsn_range().start().get();
        let member_end = member.lsn_range().end_exclusive().get();
        if start < cutoff || member_end > end {
            continue;
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
        let Ok((_, projection)) = decode_canonical_redo_v3(
            member.canonical_redo(),
            start,
            member_end,
            bound,
            None,
            limits,
            format,
        ) else {
            return false;
        };
        if projection.source_root_generation() != claim.source_root().generation() {
            continue;
        }
        if !projection.segment_updates().is_empty()
            || !projection.root_state().inline_allocations().is_empty()
            || projection.root_state().last_inline_record().is_some()
            || projection.root_state().last_inline_segment().is_some()
            || (projection.derived_retirement().is_some()
                && member.operation_identity()
                    != claim.descriptor().custody().request().idempotency())
        {
            return false;
        }
        if member.operation_identity() == claim.descriptor().custody().request().idempotency() {
            if !matches!(projection.blob_semantic(),
                PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
                    if binding.record() == claim.descriptor_record()
                        && binding.record_payload_sha256() == claim.descriptor_frame_sha256())
            {
                return false;
            }
        } else if !matches!(
            projection.blob_semantic(),
            PersistedPhysicalRecoveryBlobSemantic::None
        ) || projection.placements().iter().any(|placement| {
            !matches!(
                placement.content_class(),
                SelectedRecordContentClass::Blob(
                    BlobRecordKind::DropSetManifestV3 | BlobRecordKind::OriginalDropReserved
                )
            )
        }) {
            return false;
        }
        let mut operations = sample
            .operations()
            .iter()
            .filter(|operation| operation.idempotency_identity() == member.operation_identity());
        let Some(operation) = operations.next() else {
            return false;
        };
        if operations.next().is_some() {
            return false;
        }
        if operation.fate() == StoreRecoveryOperationFate::ProvenNoEffect {
            if !projection.placements().is_empty() {
                return false;
            }
            continue;
        }
        for placement in projection.placements() {
            if let Ok(index) =
                source.binary_search_by_key(&placement.record(), |route| route.record())
            {
                if source[index] != *placement {
                    return false;
                }
                continue;
            }
            let Ok(index) =
                projected.binary_search_by_key(&placement.record(), |route| route.record())
            else {
                return false;
            };
            if projected[index] != *placement || !witnessed.insert(placement.record()) {
                return false;
            }
        }
    }
    witnessed.len() == projected.len()
}
