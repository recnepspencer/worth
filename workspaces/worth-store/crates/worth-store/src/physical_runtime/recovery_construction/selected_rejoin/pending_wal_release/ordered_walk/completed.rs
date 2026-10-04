//! Funded actual-media rewalk for a completed C.8 history. The pending
//! release uses the same edge predicates without this completed owner.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedPendingWalReleaseBatch,
    VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
};

use super::super::super::{
    completed_history::{native_storage::HistoricalWalkStorage, CompletedHistoryHeadFold},
    control_frames::{FundedCompletedHistoricalRawSlices, SelectedControlMediaFingerprint},
    resident::StoreRejoinResidentLedger,
    SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::super::{
    addressed_root, delta, historical_chain, lineage, member_absence, ordered_history,
    ordered_released, retirement_edge,
};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, PhysicalRecoveryReadAllocation,
    StoreRecoveryBindingFreshnessSample,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_edges(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut PhysicalRecoveryReadAllocation<'_>,
    history: &VerifiedOrderedRootHistory,
    batches: &[VerifiedOrderedPendingWalReleaseBatch],
    checkpoint_generation: u64,
    checkpoint_root_sha256: [u8; 32],
    selected_root: &DurablePhysicalRootManifest,
    cutoff: u64,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    format: PhysicalRecordFormatDeclaration,
    resident: &mut StoreRejoinResidentLedger,
    raw: &mut FundedCompletedHistoricalRawSlices,
    fold: &mut CompletedHistoryHeadFold,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let first_topology = history
        .edges()
        .first()
        .ok_or(Denial::CertificateRoster)?
        .source_topology();
    if effective.ordered_replays().len() != batches.len() {
        return Err(Denial::CertificateRoster);
    }
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let mut storage = HistoricalWalkStorage::new(window, resident, raw);
    let checkpoint = addressed_root::observe_with_storage(
        &mut discovery,
        checkpoint_generation,
        selected_root.node_capacity(),
        format,
        first_topology,
        Some(checkpoint_root_sha256),
        &mut storage,
    )?;
    let checkpoint_snapshot = delta::snapshot_with_storage(
        &mut discovery,
        &checkpoint.root,
        &checkpoint.free,
        format,
        &mut storage,
    )?;
    if checkpoint_snapshot.transcript != first_topology {
        return Err(Denial::RootBinding);
    }
    let delta::Snapshot {
        routes,
        segments,
        free_entries,
        transcript: _,
        mut fingerprint,
    } = checkpoint_snapshot;
    storage.discard_vec(routes)?;
    storage.discard_vec(segments)?;
    storage.discard_vec(free_entries)?;
    fingerprint.extend_with_storage(checkpoint.fingerprint, &mut storage)?;
    let remaining = storage.resident.remaining();
    ordered_history::verify_roster_with_storage(
        history,
        first_topology,
        cutoff,
        None,
        MAX_DISCOVERY_ENTRIES,
        remaining,
        &mut storage,
    )?;
    drop(storage);
    let mut media = discovery.finish();
    let mut generation = checkpoint_generation;
    let mut topology = first_topology;
    let mut release_index = 0;
    for (edge_index, edge) in history.edges().iter().enumerate() {
        retirement_edge::verify_basis(
            history.edges(),
            edge_index,
            generation,
            cutoff,
            sample,
            || {
                member_absence::member_leaves_with_storage(
                    sample, cutoff, generation, format, window, resident,
                )
            },
        )?;
        let (next, part) = match edge {
            VerifiedOrderedRootEdge::Retirement(step) => retirement_edge::verify_completed(
                media,
                window,
                resident,
                raw,
                fold,
                step,
                generation,
                format,
                selected_root.node_capacity(),
            )?,
            VerifiedOrderedRootEdge::Ordinary(step) => {
                historical_chain::verify_completed_ordinary_step(
                    media,
                    window,
                    resident,
                    raw,
                    fold,
                    step,
                    generation,
                    topology,
                    sample,
                    frames,
                    cutoff,
                    format,
                    selected_root.node_capacity(),
                )?
            }
            VerifiedOrderedRootEdge::Released(step) => {
                let batch = batches
                    .get(release_index)
                    .ok_or(Denial::CertificateRoster)?;
                let (attached_index, replay) = effective
                    .ordered_replays()
                    .get(release_index)
                    .ok_or(Denial::CertificateRoster)?;
                if batch.edge_index() != edge_index
                    || *attached_index != edge_index
                    || !lineage::ordered_predecessor_matches(&batches[..release_index], batch)
                {
                    return Err(Denial::CertificateRoster);
                }
                release_index += 1;
                ordered_released::verify_completed(
                    media,
                    window,
                    resident,
                    raw,
                    fold,
                    step,
                    batch,
                    replay,
                    generation,
                    sample,
                    frames,
                    cutoff,
                    format,
                    selected_root.node_capacity(),
                )?
            }
        };
        let mut storage = HistoricalWalkStorage::new(window, resident, raw);
        fingerprint.extend_with_storage(part, &mut storage)?;
        drop(storage);
        media = next;
        generation = generation.checked_add(1).ok_or(Denial::BoundExceeded)?;
        topology = edge.result_topology();
    }
    if release_index != batches.len()
        || generation != selected_root.generation()
        || topology != history.selected_topology()
    {
        return Err(Denial::RootBinding);
    }
    Ok((media, fingerprint))
}
