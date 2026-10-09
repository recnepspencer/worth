//! One independently observed selected/rooted V2 media closure.

use super::*;

pub(super) struct ObservedHeadV2Rejoin {
    pub(super) selector: Vec<u8>,
    pub(super) root: Vec<u8>,
    pub(super) source_root: Vec<u8>,
    pub(super) checkpoint: Vec<u8>,
    pub(super) selected_free: DurableFreeSpaceManifestHeader,
    pub(super) selected_free_bytes: Vec<u8>,
    pub(super) source_free: Option<DurableFreeSpaceManifestHeader>,
    pub(super) source_free_bytes: Option<Vec<u8>>,
    pub(super) selected_routes: tier::routes::ResidentRouteProvenance,
    pub(super) source_routes: Option<tier::routes::ResidentRouteProvenance>,
    pub(super) root_free_slices: Vec<SelectedArtifactSlice>,
    pub(super) heads: release_heads::ObservedReleaseHeads,
    pub(super) controls: release_heads::ObservedReleaseHeadControls,
}

impl ObservedHeadV2Rejoin {
    pub(super) fn same_bytes(&self, other: &Self) -> bool {
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

    pub(super) fn into_fingerprint(
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
        let heads = SelectedControlMediaFingerprint::selected_heads(
            self.heads.into_funded_slices_with_resident(resident)?,
        );
        fingerprint.extend_with_resident(heads, resident)?;
        let selected_routes = self
            .selected_routes
            .into_media_fingerprint_with_resident(resident)?;
        fingerprint.extend_with_resident(selected_routes, resident)?;
        if let Some(source_routes) = self.source_routes {
            let source_routes = source_routes.into_media_fingerprint_with_resident(resident)?;
            fingerprint.extend_with_resident(source_routes, resident)?;
        }
        fingerprint.extend_with_resident(
            SelectedControlMediaFingerprint::observed(self.root_free_slices),
            resident,
        )?;
        Ok(fingerprint)
    }
}

pub(super) fn observe_once(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    window: &crate::physical_runtime::PhysicalRecoveryReadAllocation<'_>,
    reopen: &CompletedPhysicalRecoveryFreshReopen,
    allocation: PhysicalRecoveryAllocationAdmission,
    resident: &mut StoreRejoinResidentLedger,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    tier_claim: Option<&VerifiedSelectedTierEpochCustody>,
    checkpoint: &worth_store_physical_integrity::VerifiedCheckpointStream,
) -> Result<ObservedHeadV2Rejoin, Denial> {
    let format = reopen.format();
    let store = discovery.store_identity();
    let (selected, source) = root_checkpoint::observe_v2_with_resident(
        discovery, store, format, claim, checkpoint, resident,
    )?;
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
            let observed = tier::selection::observe(discovery, store, reopen, tier, checkpoint)?;
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
        window,
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
