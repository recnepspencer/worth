//! Exact V3 control and C.9 operation-fate joins for pending release custody.

use worth_store_physical_format::{
    BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, DropSetManifestV3, OriginalDropReservedV1,
    ReleaseCustodyHeadEntryV1, ReleasedDropPredecessorV1, ReleasedDropWalFateWitnessV1,
};

use super::super::release_custody::{
    expected_drop_key, VerifiedAddressedCheckpointReleaseBase, VerifiedSelectedCheckpointCustody,
    WitnessedSelectedControlFrame,
};
use super::PendingWalReleaseCustodyDenial;
use crate::{
    ReconciledOperationFates, RecoveryOperationFate, VerifiedSelectedReleaseHeadReplayV14,
};

#[cfg(test)]
#[path = "verification/replaced_head_tests.rs"]
mod replaced_head_tests;

pub(super) fn verify_controls(
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: &DropSetManifestV3,
    reservation_frame: &WitnessedSelectedControlFrame,
    manifest_frame: &WitnessedSelectedControlFrame,
    selected_release: Option<&VerifiedSelectedCheckpointCustody>,
    addressed_release: Option<&VerifiedAddressedCheckpointReleaseBase>,
    head_replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
) -> Result<(), PendingWalReleaseCustodyDenial> {
    let base = descriptor.base();
    let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
        return Err(PendingWalReleaseCustodyDenial::ControlBinding);
    };
    if reservation.store() != base.store()
        || reservation.reclaim_attempt() != base.reclaim_attempt()
        || reservation.manifest_record() != base.manifest_record()
        || reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
        || reservation.source_basis_digest() != base.source_basis_digest()
        || reservation.reserved_selected_generation() != base.source_root_generation()
        || reservation.request() != descriptor.custody().request()
        || manifest.store() != base.store()
        || manifest.reclaim_attempt() != base.reclaim_attempt()
        || manifest.source_basis_digest() != base.source_basis_digest()
        || manifest.count() != base.manifest_count()
        || !match (base.predecessor(), selected_release, head_replay) {
            (None, _, None) => manifest
                .dropped()
                .binary_search(&source.publication_record())
                .is_ok(),
            (Some(predecessor), Some(selected), None) => {
                let in_batch = selected.batches().iter().any(|batch| {
                    batch.descriptor_record() == predecessor.descriptor_record()
                        && batch.descriptor_frame_sha256() == predecessor.descriptor_frame_sha256()
                });
                let accumulator = selected.accumulator();
                let tip = accumulator.tip();
                let carried_tip = !accumulator.terminal()
                    && tip.descriptor_record() == predecessor.descriptor_record()
                    && tip.descriptor_frame_sha256() == predecessor.descriptor_frame_sha256()
                    && tip.candidate_root_generation() <= base.source_root_generation();
                in_batch || carried_tip
            }
            (Some(_), None, None) => addressed_release
                .is_some_and(|selected| selected.matches_predecessor(descriptor, manifest)),
            (predecessor, None, Some(replay)) => {
                let effect = replay.effect();
                let worth_store_physical_format::ReleaseCustodyHeadMutationV1::Upsert {
                    expected_prior,
                    next,
                } = effect.mutation()
                else {
                    return Err(PendingWalReleaseCustodyDenial::ControlBinding);
                };
                let key = worth_store_physical_format::ReleaseCustodyHeadKeyV1::new(
                    source.object(),
                    source.generation(),
                );
                if key != Some(next.key())
                    || effect.source_basis() != source
                    || next.source_basis_digest() != base.source_basis_digest()
                    || next.manifest_record() != base.manifest_record()
                    || next.manifest_frame_sha256() != base.manifest_frame_sha256()
                    || next.reservation_record() != reservation_frame.selected_placement().record()
                    || next.reservation_frame_sha256()
                        != reservation_frame.selected_payload_sha256()
                    || next.source_root_generation() != base.source_root_generation()
                    || next.predecessor() != predecessor
                    || next.cumulative_dropped() != base.cumulative_dropped()
                    || next.terminal() != base.terminal()
                {
                    false
                } else {
                    replaced_head_matches(
                        predecessor,
                        expected_prior,
                        next,
                        manifest
                            .dropped()
                            .binary_search(&source.publication_record())
                            .is_ok(),
                        manifest.count(),
                    )
                }
            }
            _ => false,
        }
        || manifest_frame.selected_placement().record() != base.manifest_record()
        || manifest_frame.selected_payload_sha256() != base.manifest_frame_sha256()
        || reservation_frame.selected_placement().record() == base.manifest_record()
    {
        return Err(PendingWalReleaseCustodyDenial::ControlBinding);
    }
    Ok(())
}

/// The descriptor's predecessor link fixes the head its replay replaces. A
/// first release replaces nothing and drops its own source publication. A
/// successor replaces exactly the live head its predecessor descriptor wrote
/// and advances that head's count by this manifest. A successor that replayed
/// no head, or a first release that replayed one, joins nothing.
fn replaced_head_matches(
    predecessor: Option<ReleasedDropPredecessorV1>,
    expected_prior: Option<ReleaseCustodyHeadEntryV1>,
    next: ReleaseCustodyHeadEntryV1,
    drops_source_publication: bool,
    manifest_count: u16,
) -> bool {
    match (predecessor, expected_prior) {
        (None, None) => {
            drops_source_publication && next.cumulative_dropped() == u64::from(manifest_count)
        }
        (Some(prior), Some(head)) => {
            head.key() == next.key()
                && !head.terminal()
                && head.descriptor_record() == prior.descriptor_record()
                && head.descriptor_frame_sha256() == prior.descriptor_frame_sha256()
                && head
                    .cumulative_dropped()
                    .checked_add(u64::from(manifest_count))
                    == Some(next.cumulative_dropped())
        }
        _ => false,
    }
}

pub(super) fn verify_fate(
    descriptor: BlobReclaimDescriptorV3,
    witness: ReleasedDropWalFateWitnessV1,
    fates: &ReconciledOperationFates,
    policy: [u8; 32],
    operation_id: [u8; 32],
    checkpoint_cutoff: u64,
) -> Result<RecoveryOperationFate, PendingWalReleaseCustodyDenial> {
    let request = descriptor.custody().request();
    let mut operations = fates
        .operations()
        .iter()
        .filter(|value| value.identity().idempotency() == operation_id);
    let Some(operation) = operations.next() else {
        return Err(PendingWalReleaseCustodyDenial::DurableWalFate);
    };
    if operations.next().is_some()
        || witness.lsn_start() < checkpoint_cutoff
        || operation.identity().store() != descriptor.base().store()
        || !matches!(
            operation.fate(),
            RecoveryOperationFate::AcknowledgedDurable
                | RecoveryOperationFate::DurableUnacknowledged
                | RecoveryOperationFate::Indeterminate
        )
        || operation.request_fingerprint() != request.fingerprint()
        || operation.lease_issuance_generation() != request.lease_issuance_generation()
        || operation.lease_expiry_generation() != request.lease_expiry_generation()
        || expected_drop_key(
            descriptor.base().store(),
            descriptor.base().reclaim_attempt(),
            policy,
            request,
        ) != operation_id
    {
        return Err(PendingWalReleaseCustodyDenial::DurableWalFate);
    }
    Ok(operation.fate())
}
