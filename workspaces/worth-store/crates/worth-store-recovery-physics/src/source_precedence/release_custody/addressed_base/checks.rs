//! Bounded checkpoint-source control, predecessor and retained-WAL joins.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobRecordKind, CurrentPhysicalRecordPlacement,
    OriginalDropReservationRequestV1, ReleasedDropPredecessorV1, ReleasedDropWalFateWitnessV1,
    ReleasedGenerationReclaimBasisV1, SelectedRecordContentClass,
};

use super::AddressedReleaseLineage;
use crate::source_precedence::release_custody::{
    expected_drop_key, SelectedCustodyDenial, WitnessedSelectedControlFrame,
};
use crate::{ReconciledOperationFates, RecoveryOperationFate};

pub(super) fn verify_lineage(
    prior: &[AddressedReleaseLineage],
    descriptor: BlobReclaimDescriptorV3,
    source: ReleasedGenerationReclaimBasisV1,
    count: u16,
    predecessor: Option<ReleasedDropPredecessorV1>,
) -> Result<(), SelectedCustodyDenial> {
    let base = descriptor.base();
    let previous = prior.iter().rev().find(|entry| {
        entry.source.object() == source.object() && entry.source.generation() == source.generation()
    });
    match (predecessor, previous) {
        (None, None) if base.cumulative_dropped() == u64::from(count) => Ok(()),
        (Some(predecessor), Some(previous))
            if predecessor.descriptor_record() == previous.record
                && predecessor.descriptor_frame_sha256() == previous.sha256
                && !previous.descriptor.base().terminal()
                && previous.descriptor.base().source_basis_digest()
                    == base.source_basis_digest()
                && previous
                    .descriptor
                    .base()
                    .cumulative_dropped()
                    .checked_add(u64::from(count))
                    == Some(base.cumulative_dropped()) =>
        {
            Ok(())
        }
        _ => Err(SelectedCustodyDenial::ReleaseBinding),
    }
}

pub(super) fn require_frame(
    routes: &[CurrentPhysicalRecordPlacement],
    frame: &WitnessedSelectedControlFrame,
    kind: BlobRecordKind,
) -> Result<(), SelectedCustodyDenial> {
    let placement = frame.selected_placement();
    if !matches!(placement, CurrentPhysicalRecordPlacement::Extent(extent)
        if extent.content_class() == SelectedRecordContentClass::Blob(kind))
        || !routes.iter().any(|route| *route == placement)
        || <[u8; 32]>::from(Sha256::digest(frame.bytes())) != frame.selected_payload_sha256()
    {
        return Err(SelectedCustodyDenial::SelectedControlFrame);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn require_addressed_fate(
    fates: &ReconciledOperationFates,
    frames: &[ReleasedDropWalFateWitnessV1],
    members: &[([u8; 32], u64, u64)],
    policy: [u8; 32],
    request: OriginalDropReservationRequestV1,
    store: [u8; 16],
    attempt: [u8; 16],
    witness: ReleasedDropWalFateWitnessV1,
    cutoff: u64,
) -> Result<(), SelectedCustodyDenial> {
    let denial = SelectedCustodyDenial::DurableFate;
    if witness.lsn_end_exclusive() > cutoff
        || expected_drop_key(store, attempt, policy, request) != request.idempotency()
    {
        return Err(denial);
    }
    let mut overlapping = frames.iter().filter(|frame| {
        frame.lsn_start() < witness.lsn_end_exclusive()
            && witness.lsn_start() < frame.lsn_end_exclusive()
    });
    let retained_exact = match overlapping.next() {
        Some(frame) if *frame == witness && overlapping.next().is_none() => true,
        None => false,
        _ => return Err(denial),
    };
    let mut matching_members = members
        .iter()
        .filter(|(operation, _, _)| *operation == request.idempotency());
    let retained_member = match matching_members.next() {
        Some((_, start, end))
            if *start == witness.lsn_start()
                && *end == witness.lsn_end_exclusive()
                && matching_members.next().is_none() =>
        {
            true
        }
        None => false,
        _ => return Err(denial),
    };
    let mut matching = fates
        .operations()
        .iter()
        .filter(|operation| operation.identity().idempotency() == request.idempotency());
    match matching.next() {
        Some(operation)
            if matching.next().is_none()
                && operation.identity().store() == store
                && operation.request_fingerprint() == request.fingerprint()
                && operation.lease_issuance_generation() == request.lease_issuance_generation()
                && operation.lease_expiry_generation() == request.lease_expiry_generation()
                && (matches!(
                    operation.fate(),
                    RecoveryOperationFate::AcknowledgedDurable
                        | RecoveryOperationFate::DurableUnacknowledged
                ) || (retained_exact
                    && operation.fate() == RecoveryOperationFate::Indeterminate)) =>
        {
            Ok(())
        }
        None if !retained_exact && !retained_member => Ok(()),
        _ => Err(denial),
    }
}
