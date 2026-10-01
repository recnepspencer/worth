//! Deduplicated addressed control catalog for current heads and every Batch.

use super::super::{SelectedCustodyDenial, WitnessedSelectedControlFrame};
use worth_store_physical_format::{
    verify_release_custody_head_controls_view, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1,
    BlobReclaimSourceKind, BlobRecordKind, CurrentPhysicalRecordPlacement, DropSetManifestV3View,
    OriginalDropReservedV1, PersistedRecordIdentity, ReleaseCheckpointBatchV1,
    ReleaseCustodyHeadControlIdentityV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    SelectedRecordContentClass,
};

#[derive(Debug)]
pub struct AddressedReleaseHeadControlV2 {
    descriptor: WitnessedSelectedControlFrame,
    reservation: WitnessedSelectedControlFrame,
    manifest: WitnessedSelectedControlFrame,
}

impl AddressedReleaseHeadControlV2 {
    pub fn new(
        descriptor: WitnessedSelectedControlFrame,
        reservation: WitnessedSelectedControlFrame,
        manifest: WitnessedSelectedControlFrame,
    ) -> Self {
        Self {
            descriptor,
            reservation,
            manifest,
        }
    }
    pub const fn descriptor(&self) -> &WitnessedSelectedControlFrame {
        &self.descriptor
    }
    pub const fn reservation(&self) -> &WitnessedSelectedControlFrame {
        &self.reservation
    }
    pub const fn manifest(&self) -> &WitnessedSelectedControlFrame {
        &self.manifest
    }
    pub const fn descriptor_record(&self) -> PersistedRecordIdentity {
        self.descriptor.selected_placement().record()
    }
}

pub(super) fn verify_head_controls(
    heads: &[ReleaseCustodyHeadEntryV1],
    batches: &[ReleaseCheckpointBatchV1],
    prior_cumulative_dropped: u64,
    catalog: &[AddressedReleaseHeadControlV2],
    routes: &[CurrentPhysicalRecordPlacement],
    store: [u8; 16],
) -> Result<(), SelectedCustodyDenial> {
    let denial = SelectedCustodyDenial::ReleaseBinding;
    if catalog
        .windows(2)
        .any(|pair| pair[0].descriptor_record() >= pair[1].descriptor_record())
    {
        return Err(denial);
    }
    for entry in heads {
        let triple = find(catalog, entry.descriptor_record())?;
        let (descriptor, reservation, manifest) = decode(
            triple,
            routes,
            entry.descriptor_record(),
            entry.descriptor_frame_sha256(),
            entry.reservation_record(),
            entry.reservation_frame_sha256(),
            entry.manifest_record(),
            entry.manifest_frame_sha256(),
        )?;
        let base = descriptor.base();
        let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
            return Err(denial);
        };
        let frames = ReleaseCustodyHeadControlIdentityV1::new(
            triple.descriptor.selected_placement().record(),
            triple.descriptor.selected_payload_sha256(),
            triple.reservation.selected_placement().record(),
            triple.reservation.selected_payload_sha256(),
            triple.manifest.selected_placement().record(),
            triple.manifest.selected_payload_sha256(),
        )
        .map_err(|_| denial)?;
        if verify_release_custody_head_controls_view(
            *entry,
            frames,
            descriptor,
            reservation,
            manifest,
        )
        .is_err()
            || ReleaseCustodyHeadKeyV1::new(source.object(), source.generation())
                != Some(entry.key())
            || base.store() != store
            || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || base.source_basis_digest() != entry.source_basis_digest()
            || base.predecessor() != entry.predecessor()
            || base.source_root_generation() != entry.source_root_generation()
            || base.cumulative_dropped() != entry.cumulative_dropped()
            || base.terminal() != entry.terminal()
            || !same_controls(descriptor, reservation, &manifest, store)
        {
            return Err(denial);
        }
    }
    for (index, batch) in batches.iter().enumerate() {
        let triple = find(catalog, batch.descriptor_record())?;
        let raw_descriptor =
            BlobReclaimDescriptorV3::decode(triple.descriptor.bytes()).map_err(|_| denial)?;
        let (descriptor, reservation, manifest) = decode(
            triple,
            routes,
            batch.descriptor_record(),
            batch.descriptor_frame_sha256(),
            batch.reservation_record(),
            batch.reservation_frame_sha256(),
            raw_descriptor.base().manifest_record(),
            raw_descriptor.base().manifest_frame_sha256(),
        )?;
        let prior = index
            .checked_sub(1)
            .map_or(prior_cumulative_dropped, |previous| {
                batches[previous].cumulative_dropped()
            });
        if !same_controls(descriptor, reservation, &manifest, store)
            || descriptor.custody_digest() != batch.custody_digest()
            || descriptor.custody().request() != batch.request()
            || descriptor.base().candidate_root_generation() != batch.candidate_root_generation()
            || descriptor.base().predecessor() != batch.predecessor()
            || descriptor.base().terminal() != batch.terminal()
            || batch.cumulative_dropped().checked_sub(prior) != Some(u64::from(manifest.count()))
        {
            return Err(denial);
        }
    }
    // The union catalog contains no unrelated valid triple. Current Batch
    // controls may overlap a current head and appear only once in that case.
    let overlap = batches
        .iter()
        .filter(|batch| {
            heads
                .iter()
                .any(|head| head.descriptor_record() == batch.descriptor_record())
        })
        .count();
    if catalog.len()
        != heads
            .len()
            .checked_add(batches.len())
            .and_then(|total| total.checked_sub(overlap))
            .ok_or(denial)?
    {
        return Err(denial);
    }
    Ok(())
}

fn find(
    catalog: &[AddressedReleaseHeadControlV2],
    record: PersistedRecordIdentity,
) -> Result<&AddressedReleaseHeadControlV2, SelectedCustodyDenial> {
    catalog
        .binary_search_by_key(&record, |item| item.descriptor_record())
        .map(|index| &catalog[index])
        .map_err(|_| SelectedCustodyDenial::ReleaseBinding)
}

#[allow(clippy::too_many_arguments)]
fn decode<'a>(
    triple: &'a AddressedReleaseHeadControlV2,
    routes: &[CurrentPhysicalRecordPlacement],
    descriptor_id: PersistedRecordIdentity,
    descriptor_sha: [u8; 32],
    reservation_id: PersistedRecordIdentity,
    reservation_sha: [u8; 32],
    manifest_id: PersistedRecordIdentity,
    manifest_sha: [u8; 32],
) -> Result<
    (
        BlobReclaimDescriptorV3,
        OriginalDropReservedV1,
        DropSetManifestV3View<'a>,
    ),
    SelectedCustodyDenial,
> {
    let denial = SelectedCustodyDenial::ReleaseBinding;
    for (frame, kind, record, sha) in [
        (
            &triple.descriptor,
            BlobRecordKind::ReclaimDescriptorV3,
            descriptor_id,
            descriptor_sha,
        ),
        (
            &triple.reservation,
            BlobRecordKind::OriginalDropReserved,
            reservation_id,
            reservation_sha,
        ),
        (
            &triple.manifest,
            BlobRecordKind::DropSetManifestV3,
            manifest_id,
            manifest_sha,
        ),
    ] {
        let placement = frame.selected_placement();
        if placement.record() != record
            || frame.selected_payload_sha256() != sha
            || !routes.iter().any(|route| {
                *route == placement
                    && route.content_class() == SelectedRecordContentClass::Blob(kind)
            })
        {
            return Err(denial);
        }
    }
    let descriptor =
        BlobReclaimDescriptorV3::decode(triple.descriptor.bytes()).map_err(|_| denial)?;
    let reservation =
        OriginalDropReservedV1::decode(triple.reservation.bytes()).map_err(|_| denial)?;
    let manifest = DropSetManifestV3View::decode(triple.manifest.bytes()).map_err(|_| denial)?;
    Ok((descriptor, reservation, manifest))
}

fn same_controls(
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest: &DropSetManifestV3View<'_>,
    store: [u8; 16],
) -> bool {
    let base = descriptor.base();
    base.store() == store
        && base.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
        && reservation.store() == store
        && reservation.reclaim_attempt() == base.reclaim_attempt()
        && reservation.manifest_record() == base.manifest_record()
        && reservation.manifest_frame_sha256() == base.manifest_frame_sha256()
        && reservation.source_basis_digest() == base.source_basis_digest()
        && reservation.reserved_selected_generation() == base.source_root_generation()
        && reservation.request() == descriptor.custody().request()
        && manifest.store() == store
        && manifest.reclaim_attempt() == base.reclaim_attempt()
        && manifest.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
        && manifest.source_basis().digest(store) == base.source_basis_digest()
        && manifest.count() == base.manifest_count()
        && manifest.never_reserved_slot_generation() == reservation.manifest_selected_generation()
}
