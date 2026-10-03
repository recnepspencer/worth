//! Independent Store rejoin when all postcheckpoint V3 drops are already
//! published and the selected tip has no pending release descriptor.

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
};

use super::pending_wal_release::ordered_walk;
use super::{
    control_frames::FundedCompletedHistoricalRawSlices, resident::StoreRejoinResidentLedger,
    wal_inventory, SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    SelectedWalMediaFingerprint, MAX_CLEANUP_SAMPLE_BYTES, MAX_DISCOVERY_BYTES,
    MAX_DISCOVERY_ENTRIES,
};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, IntegrityAdmittedRecoveryWalFrameView,
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCoordination,
    PhysicalRecoveryFreshnessPort, PhysicalRecoveryReadAllocation,
};

#[path = "completed_history/controls.rs"]
mod controls;
#[path = "completed_history/heads.rs"]
mod heads;
#[path = "completed_history/native_storage.rs"]
pub(super) mod native_storage;
#[path = "completed_history/selection.rs"]
mod selection;
pub(super) use heads::{
    CompletedHistoryHeadFold, CompletedHistoryHeadStep, ObservedCompletedHeadControls,
};

pub(in crate::physical_runtime::recovery_construction) fn observe_claim(
    coordination: &mut PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedOrderedHistoricalReleaseCustody,
    effective: &VerifiedEffectiveReleaseHeadRosterV14,
    allocation: PhysicalRecoveryAllocationAdmission,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        SelectedWalMediaFingerprint,
        SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    if allocation.store_identity() != media.store_identity()
        || coordination.recovery_allocation_admission() != Some(allocation)
        || claim.selected_root() != reopen.root()
    {
        return Err(Denial::RootBinding);
    }
    let mut resident =
        StoreRejoinResidentLedger::from_coordination(coordination).map_err(Denial::Resident)?;
    let checkpoint = coordination
        .require_selected_checkpoint(claim.checkpoint())
        .map_err(|_| Denial::CheckpointBinding)?;
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(coordination)
        .map_err(Denial::WalReadOwnership)?;
    let mut first = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = selection::observe(
        &mut first,
        reopen,
        claim,
        checkpoint.stream(),
        &mut window,
        &mut resident,
    )?;
    let inventory = wal_inventory::admit_complete_inventory_with_resident(
        &mut first,
        coordination,
        &mut resident,
    )
    .map_err(|denial| {
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
    let sample_charge = sample.owned_heap_bytes().ok_or(Denial::BoundExceeded)?;
    resident.retain(sample_charge).map_err(Denial::Resident)?;
    let inventory = inventory
        .into_fingerprint_with_resident(&mut resident)
        .map_err(Denial::Resident)?;
    pause_before_final_reread();
    let mut final_read = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let reread = selection::observe(
        &mut final_read,
        reopen,
        claim,
        checkpoint.stream(),
        &mut window,
        &mut resident,
    )?;
    let final_inventory = wal_inventory::admit_complete_inventory_with_resident(
        &mut final_read,
        coordination,
        &mut resident,
    )
    .map_err(|denial| {
        denial.at_resident_boundary(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission,
        )
    })?;
    if !selected.same_bytes(&reread) || !inventory.matches_reread(&final_inventory) {
        return Err(Denial::RootBinding);
    }
    selected.discard(&mut resident)?;
    inventory.discard_with_resident(&mut resident)?;
    let media = final_read.finish();
    let mut head_discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let mut head_fold = CompletedHistoryHeadFold::start(
        &mut head_discovery,
        &window,
        &reread.checkpoint_source_root,
        claim,
        effective,
        allocation,
        reopen.format(),
        &mut resident,
    )?;
    let media = head_discovery.finish();
    let mut raw = FundedCompletedHistoricalRawSlices::new(&window)?;
    let (media, mut controls_fingerprint) = controls::observe(
        media,
        &reread,
        reopen,
        claim,
        checkpoint.stream(),
        &mut window,
        &mut resident,
        &mut raw,
    )?;
    let (media, history_fingerprint) = ordered_walk::verify_historical(
        media,
        &mut window,
        claim,
        effective,
        &sample,
        final_inventory.frames(),
        reopen.format(),
        &mut resident,
        &mut raw,
        &mut head_fold,
    )?;
    let mut storage = native_storage::HistoricalWalkStorage::new(&window, &mut resident, &mut raw);
    controls_fingerprint.extend_with_storage(history_fingerprint, &mut storage)?;
    drop(storage);
    let mut final_head_discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let heads = head_fold.finish(
        &mut final_head_discovery,
        &window,
        &reread.manifest,
        effective,
        allocation,
        reopen.format(),
        &mut resident,
    )?;
    controls_fingerprint.extend(heads)?;
    let media = final_head_discovery.finish();
    reread.discard(&mut resident)?;
    drop(sample);
    resident.release(sample_charge);
    let wal_fingerprint = final_inventory
        .into_fingerprint_with_resident(&mut resident)
        .map_err(Denial::Resident)?;
    controls_fingerprint.attach_completed_history_backing(raw, &window)?;
    Ok((media, wal_fingerprint, controls_fingerprint))
}
