//! Independent Store rewalk of every root edge in an ordered pending release.
//! A typed C.8 history is only a comparison target; no edge is accepted until
//! its addressed rooted media and exact sampled C.9 member are checked here.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
    VerifiedPendingWalReleaseCustody,
};

use super::super::{
    control_frames::SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use super::{
    addressed_root, controls::Controls, delta, historical_chain, lineage, member_absence,
    ordered_history, ordered_released, retirement_edge,
};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
};

#[path = "ordered_walk/completed.rs"]
mod completed;

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    claim: &VerifiedPendingWalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    controls: &Controls,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    format: PhysicalRecordFormatDeclaration,
    retained: u64,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let history = claim.ordered_history().ok_or(Denial::CertificateRoster)?;
    let batches = claim.ordered_released_batches();
    if history.checkpoint_root_frame_sha256() != claim.checkpoint_source_root_sha256()
        || history.selected_root_frame_sha256() != claim.source_root_sha256()
        || history.selected_topology()
            != claim
                .verified_transition()
                .ok_or(Denial::RoutingFrame)?
                .source_topology()
        || batches.is_empty()
    {
        return Err(Denial::RootBinding);
    }
    verify_edges(
        media,
        window,
        history,
        batches,
        claim.checkpoint().source().root().generation(),
        claim.checkpoint_source_root_sha256(),
        claim.source_root(),
        claim
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive(),
        sample,
        frames,
        effective,
        format,
        retained,
        claim,
        controls,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_historical(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    sample: &StoreRecoveryBindingFreshnessSample,
    frames: &[IntegrityAdmittedRecoveryWalFrame],
    format: PhysicalRecordFormatDeclaration,
    resident: &mut super::super::resident::StoreRejoinResidentLedger,
    raw: &mut super::super::control_frames::FundedCompletedHistoricalRawSlices,
    fold: &mut super::super::completed_history::CompletedHistoryHeadFold,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let checkpoint_sha256 = match (claim.marker(), claim.selected_head_v2()) {
        (Some(marker), None) => marker.root_sha256(),
        (None, Some(base)) => base.source_root_sha256(),
        _ => return Err(Denial::CertificateRoster),
    };
    if claim.history().checkpoint_root_frame_sha256() != checkpoint_sha256
        || claim.history().selected_root_frame_sha256() != claim.selected_root_frame_sha256()
        || claim.released_batches().is_empty()
    {
        return Err(Denial::RootBinding);
    }
    completed::verify_edges(
        media,
        window,
        claim.history(),
        claim.released_batches(),
        claim.checkpoint().source().root().generation(),
        checkpoint_sha256,
        claim.selected_root(),
        claim
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive(),
        sample,
        frames,
        effective,
        format,
        resident,
        raw,
        fold,
    )
}

#[allow(clippy::too_many_arguments)]
fn verify_edges(
    media: AdmittedRecoveryFilesystemMedia,
    window: &mut crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
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
    retained: u64,
    claim: &VerifiedPendingWalReleaseCustody,
    controls: &Controls,
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
    let checkpoint = addressed_root::observe(
        &mut discovery,
        checkpoint_generation,
        selected_root.node_capacity(),
        format,
        first_topology,
        Some(checkpoint_root_sha256),
    )?;
    let mut remaining = delta::MAX_TRANSITION_MEMORY
        .checked_sub(retained)
        .ok_or(Denial::BoundExceeded)?;
    let checkpoint_snapshot = delta::snapshot(
        &mut discovery,
        &checkpoint.root,
        &checkpoint.free,
        format,
        &mut remaining,
    )?;
    if checkpoint_snapshot.transcript != first_topology {
        return Err(Denial::RootBinding);
    }
    let delta::Snapshot {
        routes,
        segments,
        free_entries,
        transcript: _,
        fingerprint: checkpoint_fingerprint,
    } = checkpoint_snapshot;
    // The initial inventory was authenticated above. Its placement vectors
    // are not retained while each subsequent edge allocates its own pair.
    drop((routes, segments, free_entries));
    ordered_history::verify_roster(
        history,
        first_topology,
        cutoff,
        Some((
            claim.descriptor().custody().request().idempotency(),
            claim.wal_fate().lsn_start(),
        )),
        MAX_DISCOVERY_ENTRIES,
        remaining,
    )?;
    let mut fingerprint = checkpoint_fingerprint;
    let mut media = discovery.finish();
    let mut generation = checkpoint_generation;
    let mut topology = first_topology;
    let mut release_index = 0;
    for (edge_index, edge) in history.edges().iter().enumerate() {
        let peak = retained
            .checked_add(fingerprint.retained_memory_bytes())
            .ok_or(Denial::BoundExceeded)?;
        retirement_edge::verify_basis(
            history.edges(),
            edge_index,
            generation,
            cutoff,
            sample,
            || {
                let scratch = delta::MAX_TRANSITION_MEMORY
                    .checked_sub(peak)
                    .ok_or(Denial::BoundExceeded)?;
                member_absence::member_leaves(sample, cutoff, generation, scratch, format)
            },
        )?;
        let (next, part) = match edge {
            VerifiedOrderedRootEdge::Retirement(step) => retirement_edge::verify(
                media,
                step,
                generation,
                format,
                selected_root.node_capacity(),
                peak,
            )?,
            VerifiedOrderedRootEdge::Ordinary(step) => historical_chain::verify_ordinary_steps(
                media,
                std::slice::from_ref(step),
                generation,
                topology,
                None,
                generation.checked_add(1).ok_or(Denial::BoundExceeded)?,
                step.result_topology(),
                None,
                sample,
                frames,
                cutoff,
                format,
                selected_root.node_capacity(),
                peak,
            )?,
            VerifiedOrderedRootEdge::Released(step) => {
                let batch = batches
                    .get(release_index)
                    .ok_or(Denial::CertificateRoster)?;
                let (attached_index, replay) = effective
                    .ordered_replays()
                    .get(release_index)
                    .ok_or(Denial::CertificateRoster)?;
                if batch.edge_index() != edge_index || *attached_index != edge_index {
                    return Err(Denial::CertificateRoster);
                }
                if !lineage::ordered_predecessor_matches(
                    &batches[..release_index],
                    claim
                        .selected_head_v2()
                        .map(|joined| joined.selected_heads()),
                    Some(lineage::LegacyBatchBase {
                        claim,
                        controls: controls.selected_base(),
                    }),
                    batch,
                ) {
                    return Err(Denial::ControlFrame);
                }
                release_index += 1;
                ordered_released::verify(
                    media,
                    window,
                    step,
                    batch,
                    replay,
                    generation,
                    sample,
                    frames,
                    cutoff,
                    format,
                    selected_root.node_capacity(),
                    peak,
                )?
            }
        };
        fingerprint.try_extend_bounded(
            part,
            delta::MAX_TRANSITION_MEMORY
                .checked_sub(retained)
                .ok_or(Denial::BoundExceeded)?,
        )?;
        media = next;
        generation = generation.checked_add(1).ok_or(Denial::BoundExceeded)?;
        topology = edge.result_topology();
    }
    if release_index != batches.len()
        || release_index != effective.ordered_replays().len()
        || generation != selected_root.generation()
        || topology != history.selected_topology()
    {
        return Err(Denial::RootBinding);
    }
    Ok((media, fingerprint))
}
