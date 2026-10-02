//! Carried-resident V2 route observation. The borrowed integrity view avoids
//! decoding a second owned routing block while its page frame is live.

use sha2::{Digest, Sha256};

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    ManifestBlockReference, PhysicalInventoryTranscriptBuilderV1, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile, RootRoutingBlockDecodeLimits,
    RootRoutingBlockPreflight, RootRoutingBlockScopeIdentity, RootRoutingCoordinateKey,
};
use worth_store_physical_integrity::{
    validate_root_routing_block_borrowed, BorrowedRootRoutingBlockIntegrityValidation,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

use super::{
    selected_control, validate_child, validate_leaf_count, validate_route, validate_stack_growth,
    MAX_BLOCKS,
};
use crate::physical_runtime::recovery_construction::selected_rejoin::control_frames::{
    SelectedArtifactSlice, SelectedControlMediaFingerprint,
};
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::StoreRejoinResidentLedger;
use crate::physical_runtime::recovery_construction::selected_rejoin::SelectedMediaRejoinDenial as Denial;
use crate::physical_runtime::recovery_construction::selected_rejoin::MAX_DISCOVERY_BYTES;
#[path = "resident/storage.rs"]
mod storage;
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) use storage::RouteWalkStorage;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct ResidentRouteProvenance
{
    selected_routes: Vec<CurrentPhysicalRecordPlacement>,
    slices: Vec<SelectedArtifactSlice>,
}

impl ResidentRouteProvenance {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn owned_heap_bytes(
        &self,
    ) -> Option<u64> {
        u64::try_from(self.selected_routes.capacity())
            .ok()?
            .checked_mul(
                u64::try_from(std::mem::size_of::<CurrentPhysicalRecordPlacement>()).ok()?,
            )?
            .checked_add(
                u64::try_from(self.slices.capacity()).ok()?.checked_mul(
                    u64::try_from(std::mem::size_of::<SelectedArtifactSlice>()).ok()?,
                )?,
            )
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn selected_routes(
        &self,
    ) -> &[CurrentPhysicalRecordPlacement] {
        &self.selected_routes
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_media_fingerprint_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        let route_bytes = resident
            .vector_bytes(&self.selected_routes)
            .map_err(Denial::Resident)?;
        let Self {
            selected_routes,
            slices,
        } = self;
        drop(selected_routes);
        resident.release(route_bytes);
        Ok(SelectedControlMediaFingerprint::observed(slices))
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_media_fingerprint_with_storage<
        S: RouteWalkStorage,
    >(
        self,
        storage: &mut S,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        let Self {
            selected_routes,
            slices,
        } = self;
        storage.discard_vec(selected_routes)?;
        Ok(SelectedControlMediaFingerprint::observed(slices))
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<ResidentRouteProvenance, Denial> {
    verify_inner(discovery, root, free, format, resident, None, None)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_with_storage<
    S: RouteWalkStorage,
>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<ResidentRouteProvenance, Denial> {
    verify_inner(discovery, root, free, format, storage, None, None)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_snapshot_with_storage<
    S: RouteWalkStorage,
>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    storage: &mut S,
) -> Result<(ResidentRouteProvenance, Vec<CurrentPhysicalRecordPlacement>), Denial> {
    let count = usize::try_from(root.record_count()).map_err(|_| Denial::BoundExceeded)?;
    let mut snapshot = storage.reserve_vec(count)?;
    let proof = verify_inner(
        discovery,
        root,
        free,
        format,
        storage,
        Some(transcript),
        Some(&mut snapshot),
    )?;
    Ok((proof, snapshot))
}

fn verify_inner<S: RouteWalkStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut S,
    mut transcript: Option<&mut PhysicalInventoryTranscriptBuilderV1>,
    mut snapshot: Option<&mut Vec<CurrentPhysicalRecordPlacement>>,
) -> Result<ResidentRouteProvenance, Denial> {
    const MAX_ENTRIES: u64 =
        MAX_DISCOVERY_BYTES / (4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64);
    if root.record_count() > MAX_ENTRIES {
        return Err(Denial::BoundExceeded);
    }
    let tree = PhysicalTreeIdentity::new(root.tree_identity()).ok_or(Denial::RootBinding)?;
    let mut stack = storage
        .reserve_vec::<ManifestBlockReference>(usize::from(root.routing_root().is_some()))?;
    if let Some(reference) = root.routing_root() {
        stack.push(reference);
    }
    let mut seen = VisitedNodes::default();
    let mut coordinates =
        storage.reserve_vec::<RootRoutingCoordinateKey>(if root.routing_root().is_some() {
            usize::from(root.node_capacity())
        } else {
            0
        })?;
    let mut selected_routes = Vec::new();
    let mut slices = Vec::new();
    let mut count = 0_u64;
    let mut last_record = None;
    while let Some(reference) = stack.pop() {
        if !seen.insert((reference.generation(), reference.block()), storage)? {
            return Err(Denial::RoutingFrame);
        }
        // This output is retained; admit its next slot before any page read.
        storage.grow_vec_geometrically(&mut slices, 1)?;
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let page_bytes = u64::from(format.page_size().bytes());
        let frame = storage.read_page(discovery, artifact, page_bytes)?;
        let bytes = frame.bytes().ok_or(Denial::MissingRoute)?;
        // Framing/count preflight grants no route authority; the borrowed
        // Integrity entry below still checks every semantic and root binding.
        let capacity = root.node_capacity();
        let limits = RootRoutingBlockDecodeLimits {
            leaf_entries: u64::from(capacity),
            branch_children: u64::from(capacity),
        };
        let scratch_slots = RootRoutingBlockPreflight::inspect_frame(bytes, capacity, limits)
            .map_err(|_| Denial::RoutingFrame)?
            .0
            .coordinate_scratch_slots();
        if coordinates.capacity() < scratch_slots {
            return Err(storage.overflow());
        }
        let range =
            PhysicalByteRange::new(0, bytes.len() as u64).map_err(|_| Denial::RoutingFrame)?;
        let scope = PhysicalArtifactScope::root_routing_block(
            discovery.store_identity(),
            format,
            RootRoutingBlockScopeIdentity::new(tree, reference),
            range,
        );
        let (validation, _) = validate_root_routing_block_borrowed(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
            &mut coordinates,
        )
        .map_err(|_| storage.overflow())?;
        let BorrowedRootRoutingBlockIntegrityValidation::Intact(block) = validation else {
            return Err(Denial::RoutingFrame);
        };
        slices.push(
            SelectedArtifactSlice::observed(artifact, 0, bytes, true)
                .ok_or(Denial::RoutingFrame)?,
        );
        if let Some(entries) = block.entries() {
            validate_leaf_count(entries.len(), root, count)?;
            for placement in entries {
                if !reference.contains(placement.record())
                    || last_record.is_some_and(|previous| previous >= placement.record())
                {
                    return Err(Denial::RoutingFrame);
                }
                last_record = Some(placement.record());
                validate_route(placement, free)?;
                if let Some(transcript) = transcript.as_deref_mut() {
                    transcript
                        .include_route(placement)
                        .map_err(|_| Denial::RoutingFrame)?;
                }
                if let Some(snapshot) = snapshot.as_deref_mut() {
                    snapshot.push(placement);
                }
                if selected_control(placement, false) {
                    storage.grow_vec_geometrically(&mut selected_routes, 1)?;
                    selected_routes.push(placement);
                }
                count += 1;
            }
        } else {
            let children = block.children().ok_or(Denial::RoutingFrame)?;
            if children.len() > usize::from(root.node_capacity()) {
                return Err(Denial::RoutingFrame);
            }
            validate_stack_growth(stack.len(), children.len())?;
            storage.grow_vec(&mut stack, children.len())?;
            for child in children.rev() {
                validate_child(child, reference)?;
                stack.push(child);
            }
        }
        drop(block);
        storage.discard_frame(frame)?;
    }
    if count != root.record_count() {
        return Err(Denial::RoutingFrame);
    }
    storage.discard_vec(stack)?;
    storage.discard_vec(coordinates)?;
    seen.discard(storage)?;
    Ok(ResidentRouteProvenance {
        selected_routes,
        slices,
    })
}

#[derive(Default)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct VisitedNodes {
    slots: Vec<Option<(u64, u64)>>,
    len: usize,
}

impl VisitedNodes {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn insert(
        &mut self,
        key: (u64, u64),
        storage: &mut impl RouteWalkStorage,
    ) -> Result<bool, Denial> {
        if self.len >= MAX_BLOCKS {
            return Err(Denial::RoutingFrame);
        }
        if !self.slots.is_empty() && probe_find(&self.slots, key).ok_or(Denial::BoundExceeded)? {
            return Ok(false);
        }
        if self.slots.is_empty() || (self.len + 1) * 2 > self.slots.len() {
            let capacity = self
                .slots
                .len()
                .max(8)
                .checked_mul(2)
                .ok_or(Denial::BoundExceeded)?;
            let mut replacement = storage.reserve_vec::<Option<(u64, u64)>>(capacity)?;
            replacement.resize(capacity, None);
            for old in self.slots.iter().flatten().copied() {
                if probe_insert(&mut replacement, old) != Some(true) {
                    return Err(Denial::BoundExceeded);
                }
            }
            let old = std::mem::replace(&mut self.slots, replacement);
            storage.discard_vec(old)?;
        }
        let inserted = probe_insert(&mut self.slots, key).ok_or(Denial::BoundExceeded)?;
        if inserted {
            self.len += 1;
        }
        Ok(inserted)
    }
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn discard(
        self,
        storage: &mut impl RouteWalkStorage,
    ) -> Result<(), Denial> {
        storage.discard_vec(self.slots)
    }
}

fn slot(slots: &[Option<(u64, u64)>], key: (u64, u64)) -> usize {
    let mut digest = Sha256::new();
    digest.update(key.0.to_le_bytes());
    digest.update(key.1.to_le_bytes());
    let digest: [u8; 32] = digest.finalize().into();
    (u64::from_le_bytes(digest[..8].try_into().unwrap()) as usize) % slots.len()
}

/// Exact equality decides membership. Even a hostile full-collision table
/// performs at most its admitted slot count of probes.
fn probe_insert(slots: &mut [Option<(u64, u64)>], key: (u64, u64)) -> Option<bool> {
    let mut at = slot(slots, key);
    for _ in 0..slots.len() {
        match slots[at] {
            Some(existing) if existing == key => return Some(false),
            None => {
                slots[at] = Some(key);
                return Some(true);
            }
            Some(_) => at = (at + 1) % slots.len(),
        }
    }
    None
}

fn probe_find(slots: &[Option<(u64, u64)>], key: (u64, u64)) -> Option<bool> {
    let mut at = slot(slots, key);
    for _ in 0..slots.len() {
        match slots[at] {
            Some(existing) if existing == key => return Some(true),
            None => return Some(false),
            Some(_) => at = (at + 1) % slots.len(),
        }
    }
    None
}
