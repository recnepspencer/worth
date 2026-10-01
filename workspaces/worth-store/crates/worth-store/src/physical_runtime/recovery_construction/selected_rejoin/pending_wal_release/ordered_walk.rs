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
    addressed_root, controls::Controls, delta, historical_chain, lineage, ordered_history,
    ordered_released,
};
use crate::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame, StoreRecoveryBindingFreshnessSample,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    media: AdmittedRecoveryFilesystemMedia,
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
        Some((claim, controls)),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_historical(
    media: AdmittedRecoveryFilesystemMedia,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
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
    verify_edges(
        media,
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
        retained,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn verify_edges(
    media: AdmittedRecoveryFilesystemMedia,
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
    pending: Option<(&VerifiedPendingWalReleaseCustody, &Controls)>,
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let first_topology = match history.edges().first().ok_or(Denial::CertificateRoster)? {
        VerifiedOrderedRootEdge::Ordinary(step) => step.source_topology(),
        VerifiedOrderedRootEdge::Released(edge) => edge.transition().source_topology(),
    };
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
        pending.map(|(claim, _)| {
            (
                claim.descriptor().custody().request().idempotency(),
                claim.wal_fate().lsn_start(),
            )
        }),
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
        let (next, part) = match edge {
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
                let prior = &batches[..release_index];
                if let Some((claim, controls)) = pending {
                    let matches_prior = lineage::ordered_predecessor_matches(prior, batch);
                    let has_prior_object = prior.iter().any(|earlier| {
                        earlier.manifest().source_basis() == batch.manifest().source_basis()
                    });
                    if !has_prior_object
                        && batch.descriptor().base().predecessor().is_none()
                        && lineage::selected_base_matches_source(
                            claim,
                            controls.selected_base(),
                            &batch.manifest(),
                        )
                    {
                        return Err(Denial::ControlFrame);
                    }
                    let matches_selected_base = !has_prior_object
                        && (claim.selected_release().is_some()
                            || claim.addressed_release_base().is_some())
                        && lineage::selected_base_predecessor_matches(
                            claim,
                            controls.selected_base(),
                            batch.descriptor(),
                            &batch.manifest(),
                        );
                    if !matches_prior && !matches_selected_base {
                        return Err(Denial::ControlFrame);
                    }
                } else if !lineage::ordered_predecessor_matches(prior, batch) {
                    return Err(Denial::ControlFrame);
                }
                release_index += 1;
                ordered_released::verify(
                    media,
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
        if !fingerprint.try_extend_bounded(
            part,
            delta::MAX_TRANSITION_MEMORY
                .checked_sub(retained)
                .ok_or(Denial::BoundExceeded)?,
        ) {
            return Err(Denial::BoundExceeded);
        }
        media = next;
        generation = generation.checked_add(1).ok_or(Denial::BoundExceeded)?;
        topology = match edge {
            VerifiedOrderedRootEdge::Ordinary(step) => step.result_topology(),
            VerifiedOrderedRootEdge::Released(step) => step.transition().result_topology(),
        };
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
