//! An anchored tier root is rejoined to actual selected media twice before
//! any one-shot Serving custody can be minted.

pub(super) mod no_release;
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) mod no_release_controls;
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) mod no_release_frame;
pub(super) mod released_partition;
pub(super) mod routes;
pub(super) mod selection;
pub(super) mod wal;

use std::collections::BTreeMap;
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    VerifiedSelectedNoReleaseCustody, VerifiedSelectedTierEpochCustody,
};

use super::{
    wal_inventory, SelectedMediaRejoinDenial as Denial, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_ENTRIES,
};
use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, IntegrityAdmittedRecoveryWalFrameView,
    PhysicalRecoveryCoordination, PhysicalRecoveryFreshnessPort,
};

pub(in crate::physical_runtime::recovery_construction) fn observe_claim(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    claim: &VerifiedSelectedTierEpochCustody,
    no_release_claim: Option<&VerifiedSelectedNoReleaseCustody>,
    pause_before_final_reread: impl FnOnce(),
) -> Result<
    (
        AdmittedRecoveryFilesystemMedia,
        super::SelectedWalMediaFingerprint,
        super::SelectedControlMediaFingerprint,
    ),
    Denial,
> {
    let checkpoint = coordination
        .require_selected_checkpoint(claim.checkpoint())
        .map_err(|_| Denial::CheckpointBinding)?;
    if let Some(no_release) = no_release_claim {
        coordination
            .require_selected_checkpoint(no_release.checkpoint())
            .map_err(|_| Denial::CheckpointBinding)?;
        no_release::verify_claims(claim, no_release, checkpoint.stream())?;
    }
    let store = media.store_identity();
    let mut first = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let selected = selection::observe(&mut first, store, reopen, claim, checkpoint.stream())?;
    let controls = routes::verify(
        &mut first,
        selected.root(),
        selected.free_header(),
        reopen.format(),
        no_release_claim.map(|claim| claim.checkpoint().source().identity().sequence().get()),
    )?;
    let selected_wal =
        wal_inventory::admit_complete_inventory(&mut first, coordination).map_err(|denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission,
            )
        })?;
    wal::verify(&selected_wal, claim)?;
    let media = first.finish();
    let sample = PhysicalRecoveryFreshnessPort::sample_binding(
        coordination,
        &media,
        crate::physical_runtime::StoreRecoverySamplingBasis::Checkpoint(claim.checkpoint()),
        IntegrityAdmittedRecoveryWalFrameView::from_frames(selected_wal.frames()),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        super::MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(Denial::binding_sampling)?;
    wal::verify_sample(&sample, claim)?;
    if no_release_claim.is_some() {
        no_release::verify_selected_tail(
            &sample,
            &controls,
            reopen.format(),
            selected.root().generation(),
        )?;
    }
    let selected_wal = selected_wal.into_fingerprint();
    pause_before_final_reread();
    let store = media.store_identity();
    let mut final_read = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let reread = selection::observe(&mut final_read, store, reopen, claim, checkpoint.stream())?;
    if !selected.matches_reread(&reread) {
        return Err(Denial::RootBinding);
    }
    let final_controls = routes::verify(
        &mut final_read,
        reread.root(),
        reread.free_header(),
        reopen.format(),
        no_release_claim.map(|claim| claim.checkpoint().source().identity().sequence().get()),
    )?;
    if final_controls != controls {
        return Err(Denial::ControlFrame);
    }
    let final_wal = wal_inventory::admit_complete_inventory(&mut final_read, coordination)
        .map_err(|denial| {
            denial.at_resident_boundary(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission,
            )
        })?;
    if !selected_wal.matches_reread(&final_wal) {
        return Err(Denial::WalFate);
    }
    wal::verify(&final_wal, claim)?;
    Ok((
        final_read.finish(),
        final_wal.into_fingerprint(),
        controls.into_media_fingerprint(),
    ))
}

pub(super) fn verify_failed_ingest_subset(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<super::control_frames::SelectedArtifactSlice>,
) -> Result<(), Denial> {
    verify_failed_ingest_subset_bounded(
        discovery,
        routes,
        format,
        selected_generation,
        checkpoint_sequence,
        slices,
        usize::MAX,
    )
}

pub(super) fn verify_failed_ingest_subset_bounded(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<super::control_frames::SelectedArtifactSlice>,
    max_slices: usize,
) -> Result<(), Denial> {
    no_release_controls::verify_bounded(
        discovery,
        routes,
        format,
        selected_generation,
        checkpoint_sequence,
        slices,
        max_slices,
    )?;
    Ok(())
}
