//! Store-owned second read of a fully control-joined V2 checkpoint source.
//! The C8 token is compared to actual media and does not authorize itself.

use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, BoundedRecoveryFilesystemDiscovery,
};
use worth_store_physical_format::{DurableFreeSpaceManifestHeader, BLOB_CONTROL_FRAME_MAX_BYTES};

#[path = "head_v2/resident_memory.rs"]
mod resident_memory;
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
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCoordination,
    PhysicalRecoveryFreshnessPort, PhysicalRecoveryRejoinResidentBoundary,
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
    let first = observe_once(
        &mut discovery,
        reopen,
        allocation,
        resident,
        claim,
        tier_claim,
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
        claim.checkpoint(),
        inventory.frames().iter(),
        MAX_DISCOVERY_ENTRIES,
        wal_inventory::MAX_WAL_BYTES,
        MAX_CLEANUP_SAMPLE_BYTES,
    )
    .map_err(|_| Denial::WalFate)?;
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
    pause_before_final_reread();
    let mut final_discovery = media
        .bounded_discovery(MAX_DISCOVERY_ENTRIES, MAX_DISCOVERY_BYTES)
        .map_err(Denial::Qualification)?;
    let final_read = observe_once(
        &mut final_discovery,
        reopen,
        allocation,
        resident,
        claim,
        tier_claim,
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

struct ObservedHeadV2Rejoin {
    selector: Vec<u8>,
    root: Vec<u8>,
    source_root: Vec<u8>,
    checkpoint: Vec<u8>,
    selected_free: DurableFreeSpaceManifestHeader,
    selected_free_bytes: Vec<u8>,
    source_free: Option<DurableFreeSpaceManifestHeader>,
    source_free_bytes: Option<Vec<u8>>,
    selected_routes: tier::routes::ResidentRouteProvenance,
    source_routes: Option<tier::routes::ResidentRouteProvenance>,
    root_free_slices: Vec<SelectedArtifactSlice>,
    heads: release_heads::ObservedReleaseHeads,
    controls: release_heads::ObservedReleaseHeadControls,
}

impl ObservedHeadV2Rejoin {
    fn same_bytes(&self, other: &Self) -> bool {
        self.selector == other.selector
            && self.root == other.root
            && self.source_root == other.source_root
            && self.checkpoint == other.checkpoint
            && self.selected_free == other.selected_free
            && self.selected_free_bytes == other.selected_free_bytes
            && self.source_free == other.source_free
            && self.source_free_bytes == other.source_free_bytes
            && self.selected_routes == other.selected_routes
            && self.source_routes == other.source_routes
            && self.root_free_slices == other.root_free_slices
            && self.heads.same_bytes(&other.heads)
            && self.controls.same_bytes(&other.controls)
    }

    fn into_fingerprint(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        for bytes in [
            self.selector,
            self.root,
            self.source_root,
            self.checkpoint,
            self.selected_free_bytes,
        ]
        .into_iter()
        .chain(self.source_free_bytes)
        {
            let discarded_bytes = resident.vector_bytes(&bytes).map_err(Denial::Resident)?;
            drop(bytes);
            resident.release(discarded_bytes);
        }
        let mut fingerprint = self.controls.into_fingerprint_with_resident(resident)?;
        let heads = self.heads.into_fingerprint_with_resident(resident)?;
        fingerprint
            .extend_with_resident(heads, resident)
            .map_err(Denial::Resident)?;
        let selected_routes = self
            .selected_routes
            .into_media_fingerprint_with_resident(resident)?;
        fingerprint
            .extend_with_resident(selected_routes, resident)
            .map_err(Denial::Resident)?;
        if let Some(source_routes) = self.source_routes {
            let source_routes = source_routes.into_media_fingerprint_with_resident(resident)?;
            fingerprint
                .extend_with_resident(source_routes, resident)
                .map_err(Denial::Resident)?;
        }
        fingerprint
            .extend_with_resident(
                SelectedControlMediaFingerprint::observed(self.root_free_slices),
                resident,
            )
            .map_err(Denial::Resident)?;
        Ok(fingerprint)
    }
}

fn observe_once(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    allocation: PhysicalRecoveryAllocationAdmission,
    resident: &mut StoreRejoinResidentLedger,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
) -> Result<ObservedHeadV2Rejoin, Denial> {
    let format = reopen.format();
    let store = discovery.store_identity();
    let (selected, source) =
        root_checkpoint::observe_v2_with_resident(discovery, store, format, claim, resident)?;
    let occurrence = reopen.fresh_reopen_occurrence();
    if selected.selector_bytes() != occurrence.selector().bytes()
        || selected.root_bytes() != occurrence.root().bytes()
    {
        return Err(Denial::RootBinding);
    }
    if selected.root().tree_identity() != source.root().tree_identity()
        || selected.root().tier_epoch_anchor() != source.root().tier_epoch_anchor()
        || selected.root().generation() < source.root().generation()
        || (selected.root().generation() == source.root().generation()
            && selected.root() != source.root())
        || selected.root().release_custody_head_root() != source.root().release_custody_head_root()
        || selected.root().next_release_custody_head_block()
            != source.root().next_release_custody_head_block()
    {
        return Err(Denial::RootBinding);
    }
    let (selected_free, selected_free_bytes) =
        root_free::read_header(discovery, selected.root(), format, resident)?;
    let tier_start = match tier_claim {
        Some(tier) => {
            let observed = tier::selection::observe(discovery, store, reopen, tier)?;
            if observed.root() != selected.root() || observed.free_header() != &selected_free {
                return Err(Denial::RootBinding);
            }
            selected_free.tier_epoch_start()
        }
        None if selected.root().tier_epoch_anchor().is_none()
            && selected_free.tier_epoch_start().is_none() =>
        {
            None
        }
        None => return Err(Denial::RootBinding),
    };
    let selected_routes = tier::routes::verify_with_resident(
        discovery,
        selected.root(),
        &selected_free,
        format,
        resident,
    )?;
    let (source_free, source_free_bytes, source_routes) = if source.root() == selected.root() {
        (None, None, None)
    } else {
        let (free, bytes) = root_free::read_header(discovery, source.root(), format, resident)?;
        let routes =
            tier::routes::verify_with_resident(discovery, source.root(), &free, format, resident)?;
        (Some(free), Some(bytes), Some(routes))
    };
    let control_routes = source_routes.as_ref().unwrap_or(&selected_routes);
    let control_tier_start = source_free
        .as_ref()
        .map(DurableFreeSpaceManifestHeader::tier_epoch_start)
        .unwrap_or(tier_start);
    let root_free_slices = root_free::fingerprint_slices(
        &selected,
        &selected_free_bytes,
        &source,
        source_free_bytes.as_deref(),
        resident,
    )?;
    let source_closure = ReleaseHeadCapacityCharge::selected_roster_closure_bytes(
        claim.accumulator_v2().head_count(),
        u64::from(format.page_size().bytes()),
    )
    .ok_or(Denial::BoundExceeded)?;
    let batch_closure = (claim.batches().len() as u64)
        .checked_mul(4)
        .and_then(|frames| frames.checked_mul(BLOB_CONTROL_FRAME_MAX_BYTES as u64))
        .ok_or(Denial::BoundExceeded)?;
    let root_free_memory = (selected.root_bytes().len() as u64)
        .checked_add(selected_free_bytes.len() as u64)
        .and_then(|bytes| bytes.checked_add(source.bytes().len() as u64))
        .and_then(|bytes| {
            bytes.checked_add(
                source_free_bytes
                    .as_ref()
                    .map_or(0, |free| free.len() as u64),
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                (root_free_slices.capacity() as u64)
                    .checked_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64)?,
            )
        })
        .ok_or(Denial::BoundExceeded)?;
    if source_closure
        .checked_add(batch_closure)
        .and_then(|bytes| bytes.checked_add(root_free_memory))
        .is_none_or(|bytes| bytes > allocation.byte_limit())
    {
        return Err(Denial::BoundExceeded);
    }
    let heads = release_heads::observe_with_resident(
        discovery,
        source.root(),
        format,
        allocation,
        claim.accumulator_v2().head_count(),
        claim.accumulator_v2().head_roster_digest(),
        resident,
    )?;
    if heads.entries() != claim.selected_heads() {
        return Err(Denial::CertificateRoster);
    }
    let controls = release_heads::observe_controls_on_routes_with_resident(
        discovery,
        format,
        control_routes.selected_routes(),
        control_tier_start,
        claim,
        resident,
    )?;
    let (selector, root, checkpoint) = selected.into_resident_bytes(resident)?;
    let source_root = source.into_bytes();
    Ok(ObservedHeadV2Rejoin {
        selector,
        root,
        source_root,
        checkpoint,
        selected_free,
        selected_free_bytes,
        source_free,
        source_free_bytes,
        selected_routes,
        source_routes,
        root_free_slices,
        heads,
        controls,
    })
}
