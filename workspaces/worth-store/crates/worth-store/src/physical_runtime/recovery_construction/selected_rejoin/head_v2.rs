//! Store-owned second read of a fully control-joined V2 checkpoint source.
//! The C8 token is compared to actual media and does not authorize itself.

use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, BoundedRecoveryFilesystemDiscovery,
};
use worth_store_physical_format::{DurableFreeSpaceManifestHeader, BLOB_CONTROL_FRAME_MAX_BYTES};

mod observation;
#[path = "head_v2/resident_memory.rs"]
mod resident_memory;
use observation::{observe_once, ObservedHeadV2Rejoin};
#[path = "head_v2/root_free.rs"]
mod root_free;
use worth_store_recovery_physics::{
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedTierEpochCustody,
};

use super::resident::StoreRejoinResidentLedger;
use super::{
    control_frames::SelectedArtifactSlice, release_heads, root_checkpoint, tier, wal_fate,
    wal_inventory, SelectedControlMediaFingerprint, SelectedMediaRejoinDenial as Denial,
    SelectedWalMediaFingerprint, MAX_CLEANUP_SAMPLE_BYTES, MAX_DISCOVERY_BYTES,
    MAX_DISCOVERY_ENTRIES,
};
use crate::physical_runtime::{
    durability::ReleaseHeadCapacityCharge, CompletedPhysicalRecoveryFreshReopen,
    IntegrityAdmittedRecoveryWalFrameView, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryFreshnessPort,
    PhysicalRecoveryRejoinResidentBoundary,
};

pub(in crate::physical_runtime::recovery_construction) fn observe_claim(
    coordination: &PhysicalRecoveryCoordination,
    media: AdmittedRecoveryFilesystemMedia,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    allocation: PhysicalRecoveryAllocationAdmission,
    resident: &mut StoreRejoinResidentLedger,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
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
    if tier_claim.is_some_and(|tier| {
        coordination
            .require_selected_checkpoint(tier.checkpoint())
            .is_err()
    }) {
        return Err(Denial::CheckpointBinding);
    }
    let denial = Denial::CertificateRoster;
    if allocation.store_identity() != media.store_identity()
        || coordination.recovery_allocation_admission() != Some(allocation)
        || claim.selected_root() != reopen.root()
        || claim.checkpoint().encoded_bytes() > allocation.byte_limit()
    {
        return Err(denial);
    }
    let mut discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let window =
        crate::physical_runtime::PhysicalRecoveryReadAllocation::for_coordination(coordination)
            .map_err(Denial::WalReadOwnership)?;
    let first = observe_once(
        &mut discovery,
        &window,
        reopen,
        allocation,
        resident,
        claim,
        tier_claim,
        checkpoint.stream(),
    )
    .map_err(|denial| {
        denial.at_resident_boundary(
            PhysicalRecoveryRejoinResidentBoundary::FirstSelectedMediaObservation,
        )
    })?;
    let inventory = wal_inventory::admit_complete_inventory_with_resident(
        &mut discovery,
        coordination,
        resident,
    )
    .map_err(|denial| {
        denial.at_resident_boundary(PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission)
    })?;
    if let Some(tier) = tier_claim {
        tier::wal::verify(&inventory, tier)?;
    }
    let media = discovery.finish();
    let sample = PhysicalRecoveryFreshnessPort::sample_binding(
        coordination,
        &media,
        crate::physical_runtime::StoreRecoverySamplingBasis::Checkpoint(claim.checkpoint()),
        IntegrityAdmittedRecoveryWalFrameView::from_frames(inventory.frames()),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(Denial::binding_sampling)?;
    for batch in claim.batches() {
        let attempt = first
            .controls
            .attempt_for_descriptor(batch.descriptor_record())
            .ok_or(Denial::ControlFrame)?;
        let tip = batch
            .tip_provenance()
            .map_err(|_| Denial::CertificateRoster)?;
        if !wal_fate::matches_selected_fate(
            media.store_identity(),
            attempt,
            tip,
            &sample,
            inventory.frames(),
            claim
                .checkpoint()
                .compaction_cutover()
                .wal_cutoff_lsn_exclusive(),
        ) {
            return Err(Denial::WalFate);
        }
    }
    if let Some(tier) = tier_claim {
        tier::wal::verify_sample(&sample, tier)?;
    }
    let inventory = inventory
        .into_fingerprint_with_resident(resident)
        .map_err(|cause| {
            Denial::Resident(cause)
                .at_resident_boundary(PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission)
        })?;
    pause_before_final_reread();
    let mut final_discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let final_read = observe_once(
        &mut final_discovery,
        &window,
        reopen,
        allocation,
        resident,
        claim,
        tier_claim,
        checkpoint.stream(),
    )
    .map_err(|denial| {
        denial.at_resident_boundary(
            PhysicalRecoveryRejoinResidentBoundary::FinalSelectedMediaObservation,
        )
    })?;
    if !first.same_bytes(&final_read) {
        return Err(denial);
    }
    first.discard_with_resident(resident).map_err(|denial| {
        denial.at_resident_boundary(
            PhysicalRecoveryRejoinResidentBoundary::FirstSelectedMediaObservation,
        )
    })?;
    let final_inventory = wal_inventory::admit_complete_inventory_with_resident(
        &mut final_discovery,
        coordination,
        resident,
    )
    .map_err(|denial| {
        denial.at_resident_boundary(PhysicalRecoveryRejoinResidentBoundary::FinalWalAdmission)
    })?;
    if !inventory.matches_reread(&final_inventory) {
        return Err(Denial::WalFate);
    }
    if let Some(tier) = tier_claim {
        tier::wal::verify(&final_inventory, tier)?;
    }
    inventory
        .discard_with_resident(resident)
        .map_err(|denial| {
            denial.at_resident_boundary(PhysicalRecoveryRejoinResidentBoundary::FirstWalAdmission)
        })?;
    Ok((
        final_discovery.finish(),
        final_inventory
            .into_fingerprint_with_resident(resident)
            .map_err(|cause| {
                Denial::Resident(cause).at_resident_boundary(
                    PhysicalRecoveryRejoinResidentBoundary::FingerprintHandoff,
                )
            })?,
        final_read.into_fingerprint(resident).map_err(|denial| {
            denial.at_resident_boundary(PhysicalRecoveryRejoinResidentBoundary::FingerprintHandoff)
        })?,
    ))
}
