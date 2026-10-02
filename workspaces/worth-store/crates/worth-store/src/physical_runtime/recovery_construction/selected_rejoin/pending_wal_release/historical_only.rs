//! Independent Store rejoin when all postcheckpoint V3 drops were already
//! published and an ordinary root, not another pending descriptor, is selected.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
};

use super::super::{
    wal_inventory, SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    SelectedWalMediaFingerprint, MAX_CLEANUP_SAMPLE_BYTES, MAX_DISCOVERY_BYTES,
    MAX_DISCOVERY_ENTRIES,
};
use super::{delta, ordered_walk};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, IntegrityAdmittedRecoveryWalFrameView,
    PhysicalRecoveryCoordination, PhysicalRecoveryFreshnessPort,
};

#[path = "historical_only/budget.rs"]
mod budget;
#[path = "historical_only/controls.rs"]
mod controls;
#[path = "historical_only/selection.rs"]
mod selection;

pub(in crate::physical_runtime::recovery_construction) fn observe_claim(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedWalMediaFingerprint,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let checkpoint = coordination
        .require_selected_checkpoint(claim.checkpoint())
        .map_err(|_| Denial::CheckpointBinding)?;
    let mut first = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = selection::observe(&mut first, reopen, claim, checkpoint.stream())?;
    let inventory =
        wal_inventory::admit_complete_inventory(&mut first, coordination).map_err(|denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission,
            )
        })?;
    let media = first.finish();
    let sample = PhysicalRecoveryFreshnessPort::sample_binding(
        coordination,
        &media,
        claim.checkpoint(),
        IntegrityAdmittedRecoveryWalFrameView::from_frames(inventory.frames()),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(Denial::binding_sampling)?;
    pause_before_final_reread();
    let mut final_read = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let reread = selection::observe(&mut final_read, reopen, claim, checkpoint.stream())?;
    let final_inventory = wal_inventory::admit_complete_inventory(&mut final_read, coordination)
        .map_err(|denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission,
            )
        })?;
    if !selected.same_bytes(&reread) || !inventory.matches_reread(&final_inventory) {
        return Err(Denial::RootBinding);
    }
    drop(selected);
    drop(inventory);
    let retained = retained_memory(&reread, claim, &sample, checkpoint)?;
    let (media, mut controls_fingerprint) = controls::observe(
        final_read.finish(),
        &reread,
        reopen,
        claim,
        delta::MAX_TRANSITION_MEMORY
            .checked_sub(retained)
            .ok_or(Denial::BoundExceeded)?,
        checkpoint.stream(),
    )?;
    let retained_with_controls = retained
        .checked_add(controls_fingerprint.retained_memory_bytes())
        .ok_or(Denial::BoundExceeded)?;
    let (media, history_fingerprint) = ordered_walk::verify_historical(
        media,
        &mut crate::physical_runtime::PhysicalRecoveryReadAllocation::for_coordination(
            coordination,
        )
        .map_err(Denial::WalReadOwnership)?,
        claim,
        effective,
        &sample,
        final_inventory.frames(),
        reopen.format(),
        retained_with_controls,
    )?;
    controls_fingerprint.try_extend_bounded(
        history_fingerprint,
        delta::MAX_TRANSITION_MEMORY
            .checked_sub(retained)
            .ok_or(Denial::BoundExceeded)?,
    )?;
    let wal_fingerprint = final_inventory.into_fingerprint();
    Ok((media, wal_fingerprint, controls_fingerprint))
}

fn retained_memory(
    selected: &selection::Selection,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    sample: &crate::physical_runtime::StoreRecoveryBindingFreshnessSample,
    checkpoint: &crate::physical_runtime::SharedRecoveryCheckpoint,
) -> Result<u64, Denial> {
    let checkpoint_bytes = checkpoint.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
    let edge_bytes = claim.history().edges().iter().fold(0_u64, |sum, edge| {
        let projected = match edge {
            worth_store_recovery_physics::VerifiedOrderedRootEdge::Ordinary(_) => 0,
            worth_store_recovery_physics::VerifiedOrderedRootEdge::Released(release) => {
                4 * std::mem::size_of_val(release.transition().projected()) as u64
            }
        };
        sum.saturating_add(4 * std::mem::size_of_val(edge) as u64)
            .saturating_add(projected)
    });
    let batch_bytes = claim.released_batches().iter().fold(0_u64, |sum, batch| {
        sum.saturating_add(4 * batch.retained_bytes())
    });
    let bytes = selected
        .retained_memory_bytes()
        .saturating_add(std::mem::size_of_val(claim) as u64)
        .saturating_add(checkpoint_bytes)
        .saturating_add(edge_bytes)
        .saturating_add(batch_bytes)
        .saturating_add(delta::sample_memory(sample)?)
        .saturating_add(64 << 20)
        .saturating_add(wal_inventory::MAX_WAL_BYTES + (32 << 20))
        .saturating_add(8 << 20);
    (bytes < delta::MAX_TRANSITION_MEMORY)
        .then_some(bytes)
        .ok_or(Denial::BoundExceeded)
}
