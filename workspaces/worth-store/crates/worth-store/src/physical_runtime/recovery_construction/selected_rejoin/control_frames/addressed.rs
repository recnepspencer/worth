//! Independently read every checkpoint-source Batch control, including
//! non-tip triples which may no longer survive at the final selected root.

use worth_store_physical_format::DurablePhysicalRootManifest;
use worth_store_recovery_physics::VerifiedAddressedCheckpointReleaseBase;

use super::*;

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn observe_addressed_base(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    claim: &VerifiedAddressedCheckpointReleaseBase,
    tier_epoch_start: Option<u64>,
    maximum: u64,
) -> Result<ObservedSelectedControls, Denial> {
    let tip = claim.accumulator().tip();
    let descriptor = read_one(
        discovery,
        root,
        routes,
        format,
        tip.descriptor_record(),
        BlobRecordKind::ReclaimDescriptorV3,
        tier_epoch_start,
        maximum,
    )?;
    let reservation = read_one(
        discovery,
        root,
        routes,
        format,
        tip.reservation_record(),
        BlobRecordKind::OriginalDropReserved,
        tier_epoch_start,
        maximum,
    )?;
    let manifest = read_one(
        discovery,
        root,
        routes,
        format,
        claim.tip_manifest_record(),
        BlobRecordKind::DropSetManifestV3,
        tier_epoch_start,
        maximum,
    )?;
    if descriptor.bytes() != claim.tip_descriptor_frame().bytes()
        || reservation.bytes() != claim.tip_reservation_frame().bytes()
        || manifest.bytes() != claim.tip_manifest_frame().bytes()
    {
        return Err(Denial::ControlFrame);
    }
    let tip_ids = [descriptor.record(), reservation.record(), manifest.record()];
    let mut additional = Vec::new();
    for (&record, &route) in routes {
        if tip_ids.contains(&record) {
            continue;
        }
        let SelectedRecordContentClass::Blob(kind) = route.content_class() else {
            return Err(Denial::CertificateRoster);
        };
        if !matches!(
            kind,
            BlobRecordKind::ReclaimDescriptorV3
                | BlobRecordKind::OriginalDropReserved
                | BlobRecordKind::DropSetManifestV3
        ) {
            return Err(Denial::CertificateRoster);
        }
        additional
            .try_reserve(1)
            .map_err(|_| Denial::BoundExceeded)?;
        additional.push(read_one(
            discovery,
            root,
            routes,
            format,
            record,
            kind,
            tier_epoch_start,
            maximum,
        )?);
    }
    let observed = ObservedSelectedControls {
        descriptor,
        reservation,
        manifest,
        additional,
    };
    let find = |record| {
        [
            &observed.descriptor,
            &observed.reservation,
            &observed.manifest,
        ]
        .into_iter()
        .chain(observed.additional.iter())
        .find(|frame| frame.record() == record)
    };
    for (batch, witnessed) in claim
        .batches()
        .iter()
        .take(claim.prior_controls().len())
        .zip(claim.prior_controls())
    {
        let actual_descriptor = find(batch.descriptor_record()).ok_or(Denial::ControlFrame)?;
        let actual_reservation = find(batch.reservation_record()).ok_or(Denial::ControlFrame)?;
        let BlobRecordV1::ReclaimDescriptorV3(decoded) =
            decode_blob_record(actual_descriptor.bytes()).map_err(|_| Denial::ControlFrame)?
        else {
            return Err(Denial::ControlFrame);
        };
        let actual_manifest = find(decoded.base().manifest_record()).ok_or(Denial::ControlFrame)?;
        if actual_descriptor.bytes() != witnessed.descriptor().bytes()
            || actual_reservation.bytes() != witnessed.reservation().bytes()
            || actual_manifest.bytes() != witnessed.manifest().bytes()
        {
            return Err(Denial::ControlFrame);
        }
    }
    let mut catalog = BTreeMap::new();
    for frame in [
        &observed.descriptor,
        &observed.reservation,
        &observed.manifest,
    ]
    .into_iter()
    .chain(observed.additional.iter())
    {
        if catalog.insert(frame.record(), frame).is_some() {
            return Err(Denial::ControlFrame);
        }
    }
    catalog::verify_addressed(claim, &catalog)?;
    Ok(observed)
}

fn read_one(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
    tier_epoch_start: Option<u64>,
    maximum: u64,
) -> Result<ObservedSelectedControlFrame, Denial> {
    let CurrentPhysicalRecordPlacement::Extent(placement) =
        *routes.get(&record).ok_or(Denial::ControlFrame)?
    else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    if placement.content_class() != SelectedRecordContentClass::Blob(kind)
        || root.tier_epoch_anchor().is_some() != tier_epoch_start.is_some()
        || placement.tier_class()
            != arena_tier_at_epoch(tier_epoch_start, placement.arena_range().arena())
        || placement.payload_bytes() == 0
        || placement.payload_bytes() > maximum
    {
        return Err(Denial::ControlFrame);
    }
    let mut slices = Vec::new();
    let (bytes, witness) = read_extent(discovery, format, placement, &mut slices)?;
    let decoded = decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?;
    if decoded.kind() != kind || !witness.matches_frame(&bytes) {
        return Err(Denial::ControlFrame);
    }
    let frame_sha256 = Sha256::digest(&bytes).into();
    Ok(ObservedSelectedControlFrame {
        placement,
        kind,
        bytes,
        frame_sha256,
        slices,
    })
}
