//! Independently rereads the exact checkpoint-source control closure for
//! current heads and intermediate checkpoint Batches. C8's token does not
//! substitute for these Store-owned selected-media witnesses.

use std::collections::BTreeMap;

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    arena_tier_at_epoch, BlobReclaimDescriptorV3, BlobRecordKind, CurrentPhysicalRecordPlacement,
    DropSetManifestV3View, OriginalDropReservedV1, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{
    AddressedReleaseHeadControlV2, VerifiedSelectedReleaseHeadCustodyV2,
    WitnessedSelectedControlFrame,
};

use super::super::control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint};
use super::super::resident::{PhysicalRecoveryRejoinResidentDenial, StoreRejoinResidentLedger};
use super::super::SelectedMediaRejoinDenial as Denial;
mod allocation;
use allocation::ControlClosureAllocation;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct ObservedReleaseHeadControls
{
    catalog: Vec<AddressedReleaseHeadControlV2>,
    slices: Vec<SelectedArtifactSlice>,
}

impl ObservedReleaseHeadControls {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn owned_heap_bytes(
        &self,
    ) -> Option<u64> {
        let catalog = u64::try_from(self.catalog.capacity()).ok()?.checked_mul(
            u64::try_from(std::mem::size_of::<AddressedReleaseHeadControlV2>()).ok()?,
        )?;
        let slices = u64::try_from(self.slices.capacity())
            .ok()?
            .checked_mul(u64::try_from(std::mem::size_of::<SelectedArtifactSlice>()).ok()?)?;
        self.catalog
            .iter()
            .try_fold(catalog.checked_add(slices)?, |total, triple| {
                [triple.descriptor(), triple.reservation(), triple.manifest()]
                    .into_iter()
                    .try_fold(total, |subtotal, frame| {
                        subtotal.checked_add(frame.owned_heap_bytes()?)
                    })
            })
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn retained_memory_bytes(
        &self,
    ) -> u64 {
        let backing = (std::mem::size_of::<Self>() as u64)
            .saturating_add(
                (self.catalog.capacity() as u64)
                    .saturating_mul(std::mem::size_of::<AddressedReleaseHeadControlV2>() as u64),
            )
            .saturating_add(
                (self.slices.capacity() as u64)
                    .saturating_mul(std::mem::size_of::<SelectedArtifactSlice>() as u64),
            );
        self.catalog
            .iter()
            .flat_map(|triple| [triple.descriptor(), triple.reservation(), triple.manifest()])
            .fold(backing, |bytes, frame| {
                bytes.saturating_add(frame.owned_heap_bytes().unwrap_or(u64::MAX))
            })
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn attempt_for_descriptor(
        &self,
        record: PersistedRecordIdentity,
    ) -> Option<[u8; 16]> {
        self.catalog.iter().find_map(|triple| {
            (triple.descriptor_record() == record)
                .then(|| BlobReclaimDescriptorV3::decode(triple.descriptor().bytes()).ok())
                .flatten()
                .map(|descriptor| descriptor.base().reclaim_attempt())
        })
    }

    /// Every selected record in the witnessed closure, three per control set.
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn control_records(
        &self,
    ) -> impl Iterator<Item = PersistedRecordIdentity> + '_ {
        self.catalog.iter().flat_map(|triple| {
            [
                triple.descriptor_record(),
                triple.reservation_record(),
                triple.manifest_record(),
            ]
        })
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn control_record_count(
        &self,
    ) -> Option<usize> {
        self.catalog.len().checked_mul(3)
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn same_bytes(
        &self,
        other: &Self,
    ) -> bool {
        self.slices == other.slices
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_fingerprint(
        self,
    ) -> SelectedControlMediaFingerprint {
        SelectedControlMediaFingerprint::observed(self.slices)
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_fingerprint_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        self.into_slices_with_resident(resident)
            .map(SelectedControlMediaFingerprint::observed)
    }

    /// Releases the decoded catalog and keeps the resident-funded slices.
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_slices_with_resident(
        self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Vec<SelectedArtifactSlice>, Denial> {
        let catalog_heap = u64::try_from(self.catalog.capacity())
            .ok()
            .and_then(|capacity| {
                capacity.checked_mul(std::mem::size_of::<AddressedReleaseHeadControlV2>() as u64)
            })
            .and_then(|backing| {
                self.catalog.iter().try_fold(backing, |total, triple| {
                    [triple.descriptor(), triple.reservation(), triple.manifest()]
                        .into_iter()
                        .try_fold(total, |subtotal, frame| {
                            subtotal.checked_add(frame.owned_heap_bytes()?)
                        })
                })
            })
            .ok_or_else(|| {
                Denial::Resident(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                    admitted: resident.used().saturating_add(resident.remaining()),
                })
            })?;
        let Self { catalog, slices } = self;
        drop(catalog);
        resident.release(catalog_heap);
        Ok(slices)
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn observe_controls(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    source_routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    tier_epoch_start: Option<u64>,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    recovery_budget_bytes: u64,
) -> Result<ObservedReleaseHeadControls, Denial> {
    observe_control_closure(
        discovery,
        format,
        source_routes,
        tier_epoch_start,
        claim,
        ControlClosureAllocation::Pending {
            maximum: recovery_budget_bytes,
            payloads: 0,
        },
    )
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn observe_controls_on_routes_with_resident(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    source_routes: &[CurrentPhysicalRecordPlacement],
    tier_epoch_start: Option<u64>,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<ObservedReleaseHeadControls, Denial> {
    observe_control_closure_on_lookup(
        discovery,
        format,
        RouteLookup::Ordered(source_routes),
        tier_epoch_start,
        claim,
        ControlClosureAllocation::Rejoin(resident),
    )
}

#[derive(Clone, Copy)]
enum RouteLookup<'a> {
    Legacy(&'a BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>),
    Ordered(&'a [CurrentPhysicalRecordPlacement]),
}

impl RouteLookup<'_> {
    fn get(self, record: PersistedRecordIdentity) -> Option<CurrentPhysicalRecordPlacement> {
        match self {
            Self::Legacy(routes) => routes.get(&record).copied(),
            Self::Ordered(routes) => routes
                .binary_search_by_key(&record, |route| route.record())
                .ok()
                .map(|index| routes[index]),
        }
    }
}

fn observe_control_closure(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    source_routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    tier_epoch_start: Option<u64>,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    allocation: ControlClosureAllocation<'_>,
) -> Result<ObservedReleaseHeadControls, Denial> {
    observe_control_closure_on_lookup(
        discovery,
        format,
        RouteLookup::Legacy(source_routes),
        tier_epoch_start,
        claim,
        allocation,
    )
}

fn observe_control_closure_on_lookup(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    source_routes: RouteLookup<'_>,
    tier_epoch_start: Option<u64>,
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    mut allocation: ControlClosureAllocation<'_>,
) -> Result<ObservedReleaseHeadControls, Denial> {
    let needed = selected_control_keys(claim, &mut allocation)?;
    let mut catalog = allocation.reserve(needed.len())?;
    let mut slices = Vec::new();
    for &(descriptor_id, reservation_id) in &needed {
        let descriptor = read_control(
            discovery,
            format,
            source_routes,
            tier_epoch_start,
            descriptor_id,
            BlobRecordKind::ReclaimDescriptorV3,
            &mut slices,
            &mut allocation,
        )?;
        let decoded = BlobReclaimDescriptorV3::decode(descriptor.bytes())
            .map_err(|_| Denial::ControlFrame)?;
        let manifest_id = decoded.base().manifest_record();
        let reservation = read_control(
            discovery,
            format,
            source_routes,
            tier_epoch_start,
            reservation_id,
            BlobRecordKind::OriginalDropReserved,
            &mut slices,
            &mut allocation,
        )?;
        let manifest = read_control(
            discovery,
            format,
            source_routes,
            tier_epoch_start,
            manifest_id,
            BlobRecordKind::DropSetManifestV3,
            &mut slices,
            &mut allocation,
        )?;
        catalog.push(AddressedReleaseHeadControlV2::new(
            descriptor,
            reservation,
            manifest,
        ));
    }
    allocation.discard(needed)?;
    match source_routes {
        RouteLookup::Legacy(routes) => {
            let mut ordered_routes = allocation.reserve(routes.len())?;
            ordered_routes.extend(routes.values().copied());
            claim
                .revalidate_controls(&ordered_routes, &catalog)
                .map_err(|_| Denial::ControlFrame)?;
            allocation.discard(ordered_routes)?;
        }
        RouteLookup::Ordered(routes) => claim
            .revalidate_controls(routes, &catalog)
            .map_err(|_| Denial::ControlFrame)?,
    }
    Ok(ObservedReleaseHeadControls { catalog, slices })
}

fn selected_control_keys(
    claim: &VerifiedSelectedReleaseHeadCustodyV2,
    allocation: &mut ControlClosureAllocation<'_>,
) -> Result<Vec<(PersistedRecordIdentity, PersistedRecordIdentity)>, Denial> {
    let count = claim
        .selected_heads()
        .len()
        .checked_add(claim.batches().len())
        .ok_or(Denial::BoundExceeded)?;
    let mut needed = allocation.reserve(count)?;
    needed.extend(
        claim
            .selected_heads()
            .iter()
            .map(|head| (head.descriptor_record(), head.reservation_record())),
    );
    needed.extend(
        claim
            .batches()
            .iter()
            .map(|batch| (batch.descriptor_record(), batch.reservation_record())),
    );
    needed.sort_unstable();
    if needed
        .windows(2)
        .any(|pair| pair[0].0 == pair[1].0 && pair[0].1 != pair[1].1)
    {
        return Err(Denial::ControlFrame);
    }
    needed.dedup();
    Ok(needed)
}

#[allow(clippy::too_many_arguments)]
fn read_control(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    routes: RouteLookup<'_>,
    tier_epoch_start: Option<u64>,
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
    slices: &mut Vec<SelectedArtifactSlice>,
    allocation: &mut ControlClosureAllocation<'_>,
) -> Result<WitnessedSelectedControlFrame, Denial> {
    let route = routes.get(record).ok_or(Denial::MissingRoute)?;
    if route.content_class() != SelectedRecordContentClass::Blob(kind) {
        return Err(Denial::ControlFrame);
    }
    let CurrentPhysicalRecordPlacement::Extent(placement) = route else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    if placement.payload_bytes() == 0
        || placement.payload_bytes() > BLOB_CONTROL_FRAME_MAX_BYTES as u64
        || placement.tier_class()
            != arena_tier_at_epoch(tier_epoch_start, placement.arena_range().arena())
    {
        return Err(Denial::ControlFrame);
    }
    let (bytes, witness) = allocation.read_payload(discovery, format, placement, slices)?;
    match kind {
        BlobRecordKind::ReclaimDescriptorV3 => {
            BlobReclaimDescriptorV3::decode(&bytes).map_err(|_| Denial::ControlFrame)?;
        }
        BlobRecordKind::OriginalDropReserved => {
            OriginalDropReservedV1::decode(&bytes).map_err(|_| Denial::ControlFrame)?;
        }
        BlobRecordKind::DropSetManifestV3 => {
            DropSetManifestV3View::decode(&bytes).map_err(|_| Denial::ControlFrame)?;
        }
        _ => return Err(Denial::ControlFrame),
    }
    WitnessedSelectedControlFrame::from_validated(bytes, witness).map_err(|_| Denial::ControlFrame)
}
