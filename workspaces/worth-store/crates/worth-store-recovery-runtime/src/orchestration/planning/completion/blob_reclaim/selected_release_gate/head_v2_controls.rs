//! Bounded actual-media witnesses for the checkpoint-source V2 head and Batch roster.
//! Only physics may turn these bytes into selected release authority.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
    BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{
    AddressedReleaseHeadControlV2, VerifiedCheckpointReleaseHeadRosterV2,
    WitnessedSelectedControlFrame,
};

use super::super::record;
use crate::{
    entry::PhysicalRecoveryReleaseHeadControlDenial as Denial,
    integrity_ingress::RecoveryIntegrityIngressTrace,
    orchestration::planning::{
        manifest_entry_budget::ManifestEntryBudget, selected_source_inventory::ResidentAllowance,
    },
};

// read_with_witness holds the selected manifest, one chunk frame, assembled
// payload, and integrity witness concurrently before returning the payload.
const CONTROL_READ_WITNESS_OVERHEAD_BYTES: u64 = 1024;

#[derive(Clone, Copy)]
struct RequestedTriple {
    descriptor: PersistedRecordIdentity,
    descriptor_sha256: [u8; 32],
    reservation: PersistedRecordIdentity,
    reservation_sha256: [u8; 32],
    manifest: Option<(PersistedRecordIdentity, [u8; 32])>,
}

/// The source routes must be the integrity-admitted, record-sorted inventory
/// of the checkpoint-source root. The returned catalog is descriptor-sorted;
/// overlapping current Batch and head requests share one media witness.
#[allow(clippy::too_many_arguments)]
pub(super) fn read_checkpoint_source_controls(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &[CurrentPhysicalRecordPlacement],
    roster: &VerifiedCheckpointReleaseHeadRosterV2,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut ResidentAllowance,
) -> Result<Vec<AddressedReleaseHeadControlV2>, Denial> {
    if routes
        .windows(2)
        .any(|pair| pair[0].record() >= pair[1].record())
    {
        return Err(Denial::RouteOrder);
    }
    if roster
        .selected_heads()
        .windows(2)
        .any(|pair| pair[0].key() >= pair[1].key())
    {
        return Err(Denial::HeadOrder);
    }
    let request_count = roster
        .selected_heads()
        .len()
        .checked_add(roster.batches().len())
        .ok_or(Denial::RequestCountOverflow)?;
    resident
        .entries(request_count, std::mem::size_of::<RequestedTriple>())
        .map_err(|_| resident_failure(resident))?;
    let mut requests = Vec::new();
    let requested_bytes = request_bytes::<RequestedTriple>(request_count)?;
    requests
        .try_reserve_exact(request_count)
        .map_err(|cause| Denial::Allocation {
            requested: requested_bytes,
            cause,
        })?;
    for head in roster.selected_heads() {
        requests.push(RequestedTriple {
            descriptor: head.descriptor_record(),
            descriptor_sha256: head.descriptor_frame_sha256(),
            reservation: head.reservation_record(),
            reservation_sha256: head.reservation_frame_sha256(),
            manifest: Some((head.manifest_record(), head.manifest_frame_sha256())),
        });
    }
    for batch in roster.batches() {
        requests.push(RequestedTriple {
            descriptor: batch.descriptor_record(),
            descriptor_sha256: batch.descriptor_frame_sha256(),
            reservation: batch.reservation_record(),
            reservation_sha256: batch.reservation_frame_sha256(),
            manifest: None,
        });
    }
    requests.sort_unstable_by_key(|request| request.descriptor);
    let mut unique: Vec<RequestedTriple> = Vec::new();
    resident
        .entries(request_count, std::mem::size_of::<RequestedTriple>())
        .map_err(|_| resident_failure(resident))?;
    let requested_bytes = request_bytes::<RequestedTriple>(request_count)?;
    unique
        .try_reserve_exact(request_count)
        .map_err(|cause| Denial::Allocation {
            requested: requested_bytes,
            cause,
        })?;
    for request in requests {
        if let Some(previous) = unique.last_mut() {
            if previous.descriptor == request.descriptor {
                if previous.descriptor_sha256 != request.descriptor_sha256
                    || previous.reservation != request.reservation
                    || previous.reservation_sha256 != request.reservation_sha256
                    || previous.manifest.is_some()
                        && request.manifest.is_some()
                        && previous.manifest != request.manifest
                {
                    return Err(Denial::ConflictingRequest {
                        descriptor: request.descriptor,
                    });
                }
                if previous.manifest.is_none() {
                    previous.manifest = request.manifest;
                }
                continue;
            }
        }
        unique.push(request);
    }
    resident
        .entries(
            unique.len(),
            std::mem::size_of::<AddressedReleaseHeadControlV2>(),
        )
        .map_err(|_| resident_failure(resident))?;
    let mut controls = Vec::new();
    let requested_bytes = request_bytes::<AddressedReleaseHeadControlV2>(unique.len())?;
    controls
        .try_reserve_exact(unique.len())
        .map_err(|cause| Denial::Allocation {
            requested: requested_bytes,
            cause,
        })?;
    for request in unique {
        let descriptor = read_control(
            discovery,
            routes,
            request.descriptor,
            BlobRecordKind::ReclaimDescriptorV3,
            request.descriptor_sha256,
            format,
            budget,
            trace,
            scratch,
            resident,
        )?;
        let BlobRecordV1::ReclaimDescriptorV3(decoded) = decode_blob_record(descriptor.bytes())
            .map_err(|denial| Denial::DescriptorDecode {
                record: request.descriptor,
                denial,
            })?
        else {
            return Err(Denial::DescriptorKind {
                record: request.descriptor,
            });
        };
        let manifest = (
            decoded.base().manifest_record(),
            decoded.base().manifest_frame_sha256(),
        );
        if request
            .manifest
            .is_some_and(|expected| expected != manifest)
            || request.descriptor == request.reservation
            || request.descriptor == manifest.0
            || request.reservation == manifest.0
        {
            return Err(Denial::DescriptorManifestMismatch {
                record: request.descriptor,
            });
        }
        let reservation = read_control(
            discovery,
            routes,
            request.reservation,
            BlobRecordKind::OriginalDropReserved,
            request.reservation_sha256,
            format,
            budget,
            trace,
            scratch,
            resident,
        )?;
        let manifest = read_control(
            discovery,
            routes,
            manifest.0,
            BlobRecordKind::DropSetManifestV3,
            manifest.1,
            format,
            budget,
            trace,
            scratch,
            resident,
        )?;
        controls.push(AddressedReleaseHeadControlV2::new(
            descriptor,
            reservation,
            manifest,
        ));
    }
    Ok(controls)
}

fn request_bytes<T>(count: usize) -> Result<u64, Denial> {
    u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
        .ok_or(Denial::RequestCountOverflow)
}

/// The allowance that refused: the resident one, which holds its counts for
/// the block's cause, or else the entry budget, which holds its own.
fn resident_failure(resident: &ResidentAllowance) -> Denial {
    if resident.exceeded_requirement().is_some() {
        Denial::ResidentBoundExceeded
    } else {
        Denial::ManifestEntryLimit
    }
}

#[allow(clippy::too_many_arguments)]
fn read_control(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &[CurrentPhysicalRecordPlacement],
    record_id: PersistedRecordIdentity,
    kind: BlobRecordKind,
    expected_sha256: [u8; 32],
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut ResidentAllowance,
) -> Result<WitnessedSelectedControlFrame, Denial> {
    let index = routes
        .binary_search_by_key(&record_id, |route| route.record())
        .map_err(|_| Denial::RouteMissing { record: record_id })?;
    let route = routes[index];
    if !matches!(route, CurrentPhysicalRecordPlacement::Extent(extent)
        if extent.content_class() == SelectedRecordContentClass::Blob(kind)
            && extent.payload_bytes() > 0
            && extent.payload_bytes() <= BLOB_CONTROL_FRAME_MAX_BYTES as u64)
    {
        return Err(Denial::RouteMismatch { record: record_id });
    }
    // Payload, admitted chunk frame, decoded control, and returned witness
    // coexist while the selected control is read. Debit before any allocation.
    resident
        .bytes(route.payload_bytes())
        .map_err(|_| resident_failure(resident))?;
    resident
        .transient(
            route
                .payload_bytes()
                .checked_mul(3)
                .and_then(|v| v.checked_add(u64::from(format.page_size().bytes()).checked_mul(3)?))
                .and_then(|v| v.checked_add(CONTROL_READ_WITNESS_OVERHEAD_BYTES))
                .ok_or(Denial::RequestCountOverflow)?,
        )
        .map_err(|_| resident_failure(resident))?;
    let (bytes, witness) = record::read_with_witness_diagnostic(
        discovery,
        format,
        Some(route),
        record_id,
        BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        budget,
        trace,
        scratch,
        resident,
    )
    .map_err(|denial| Denial::ControlRead {
        record: record_id,
        denial,
    })?;
    if <[u8; 32]>::from(Sha256::digest(&bytes)) != expected_sha256 {
        return Err(Denial::FrameDigestMismatch { record: record_id });
    }
    resident
        .transient(u64::try_from(bytes.len()).map_err(|_| Denial::RequestCountOverflow)?)
        .map_err(|_| resident_failure(resident))?;
    WitnessedSelectedControlFrame::from_validated(bytes, witness).map_err(|denial| {
        Denial::WitnessMismatch {
            record: record_id,
            denial,
        }
    })
}
